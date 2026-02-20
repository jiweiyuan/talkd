use crate::crypto::{derive_topic, generate_daemon_id, load_or_create_identity};
use crate::protocol::*;
use crate::store::Store;
use anyhow::Result;
use bytes::Bytes;
use iroh::endpoint::Endpoint;
use iroh::protocol::Router;
use iroh::{EndpointAddr, EndpointId, RelayMode};
use iroh_gossip::net::{Gossip, GOSSIP_ALPN};
use iroh_gossip::proto::TopicId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{Mutex, Notify};
use tracing::{debug, error, info, warn};

// ── Types ───────────────────────────────────────────────────────────

struct Subscriber {
    notify: Arc<Notify>,
}

struct Channel {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    topic_id: TopicId,
    gossip_sender: Option<iroh_gossip::api::GossipSender>,
    peers: HashMap<String, EndpointId>, // short_id → full NodeId
    subscribers: HashMap<String, Subscriber>,
}

pub struct Daemon {
    id: String,
    secret_key: iroh::SecretKey,
    channels: HashMap<String, Channel>,
    endpoint: Endpoint,
    gossip: Gossip,
    store: Store,
}

// ── Paths ───────────────────────────────────────────────────────────

pub fn talkd_dir() -> PathBuf {
    std::env::var("TALKD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or("/tmp".into()).join(".talkd"))
}
fn socket_path() -> PathBuf {
    talkd_dir().join("daemon.sock")
}
fn pid_path() -> PathBuf {
    talkd_dir().join("daemon.pid")
}
fn log_path() -> PathBuf {
    talkd_dir().join("daemon.log")
}

fn setup_file_logging() {
    use tracing_subscriber::{fmt, EnvFilter};
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .expect("Cannot open log file");
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("talkd=debug".parse().unwrap()),
        )
        .with_writer(std::sync::Mutex::new(log_file))
        .with_ansi(false)
        .init();
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

// ── Main daemon entry ───────────────────────────────────────────────

pub async fn run_daemon() -> Result<()> {
    let dir = talkd_dir();
    std::fs::create_dir_all(&dir)?;
    setup_file_logging();
    std::fs::write(pid_path(), std::process::id().to_string())?;
    let _ = std::fs::remove_file(socket_path());

    // ── Persistent identity ─────────────────────────────────────
    let secret_key = load_or_create_identity(&dir)?;
    let secret_key_copy = iroh::SecretKey::from_bytes(&secret_key.to_bytes());
    info!("Identity loaded: {}", secret_key.public());

    // ── iroh Endpoint ───────────────────────────────────────────
    // Pkarr discovery is included by default — our NodeId is
    // automatically published to the BT DHT and resolvable by
    // anyone who knows our NodeId. No extra code needed.
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(RelayMode::Default)
        .bind()
        .await?;

    endpoint.online().await;
    let my_id = endpoint.id();
    info!("iroh endpoint ready: {}", my_id.fmt_short());

    // ── iroh-gossip ─────────────────────────────────────────────
    // 4MB max message size (default 4KB is too small for file attachments)
    let gossip = Gossip::builder()
        .max_message_size(4 * 1024 * 1024)
        .spawn(endpoint.clone());

    // ── Router (accept incoming gossip + DM connections) ────────
    let _router = Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .spawn();
    info!("Router started (gossip ALPN registered)");

    // ── Store ───────────────────────────────────────────────────
    let store = Store::open(&dir)?;

    let daemon_id = generate_daemon_id();
    info!("Daemon started id={} pid={}", daemon_id, std::process::id());

    let daemon = Arc::new(Mutex::new(Daemon {
        id: daemon_id,
        secret_key: secret_key_copy,
        channels: HashMap::new(),
        endpoint,
        gossip,
        store,
    }));

    // ── IPC server ──────────────────────────────────────────────
    let ipc = UnixListener::bind(socket_path())?;
    info!("IPC listening on {}", socket_path().display());

    loop {
        match ipc.accept().await {
            Ok((stream, _)) => {
                let d = daemon.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_ipc(stream, d).await {
                        debug!("IPC error: {}", e);
                    }
                });
            }
            Err(e) => error!("IPC accept error: {}", e),
        }
    }
}

// ── IPC Handler ─────────────────────────────────────────────────────

async fn handle_ipc(
    stream: tokio::net::UnixStream,
    daemon: Arc<Mutex<Daemon>>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let cmd: Command = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                reply(&mut writer, &Response::err(format!("Bad command: {}", e))).await?;
                continue;
            }
        };

        match cmd {
            Command::Join { channel, client_id } => {
                let resp = {
                    let mut d = daemon.lock().await;
                    d.join_channel(&channel, &client_id).await
                };
                reply(&mut writer, &resp).await?;
            }

            Command::Send {
                channel,
                message,
                client_id,
                file,
            } => {
                let resp = {
                    let mut d = daemon.lock().await;
                    d.send_message(&channel, &message, &client_id, file.as_ref()).await
                };
                reply(&mut writer, &resp).await?;
            }

            Command::Dm {
                target,
                message,
                client_id,
                file,
            } => {
                let resp = {
                    let mut d = daemon.lock().await;
                    d.send_dm(&target, &message, &client_id, file.as_ref()).await
                };
                reply(&mut writer, &resp).await?;
            }

            Command::Read {
                channel,
                client_id,
                wait,
                timeout,
            } => {
                let notify = {
                    let mut d = daemon.lock().await;
                    match d.channels.get_mut(&channel) {
                        None => {
                            reply(
                                &mut writer,
                                &Response::err(format!("Not in channel: {}", channel)),
                            )
                            .await?;
                            continue;
                        }
                        Some(ch) => {
                            if !ch.subscribers.contains_key(&client_id) {
                                ch.subscribers.insert(
                                    client_id.clone(),
                                    Subscriber {
                                        notify: Arc::new(Notify::new()),
                                    },
                                );
                            }
                            ch.subscribers.get(&client_id).unwrap().notify.clone()
                        }
                    }
                };

                // Immediate read (no wait)
                if !wait {
                    let d = daemon.lock().await;
                    let msgs = d.store.read_messages(&channel, &client_id).unwrap_or_default();
                    let mut resp = Response::ok();
                    resp.messages = Some(msgs);
                    reply(&mut writer, &resp).await?;
                    continue;
                }

                // --wait: loop until messages arrive or timeout
                let deadline = timeout.map(|secs| {
                    tokio::time::Instant::now() + std::time::Duration::from_secs(secs)
                });

                let msgs = loop {
                    // Check for messages
                    {
                        let d = daemon.lock().await;
                        let msgs = d.store.read_messages(&channel, &client_id).unwrap_or_default();
                        if !msgs.is_empty() {
                            break msgs;
                        }
                    }

                    // Register THEN wait for notification
                    let notified = notify.notified();

                    if let Some(dl) = deadline {
                        let remaining = dl.saturating_duration_since(tokio::time::Instant::now());
                        if remaining.is_zero() {
                            break vec![]; // timeout
                        }
                        if tokio::time::timeout(remaining, notified).await.is_err() {
                            // Timeout — one final read attempt
                            let d = daemon.lock().await;
                            break d.store.read_messages(&channel, &client_id).unwrap_or_default();
                        }
                    } else {
                        notified.await;
                    }
                    // Notification received — loop back to read from disk
                };

                let mut resp = Response::ok();
                resp.messages = Some(msgs);
                reply(&mut writer, &resp).await?;
            }

            Command::Leave { channel, client_id } => {
                let mut d = daemon.lock().await;
                if let Some(ch) = d.channels.get_mut(&channel) {
                    ch.subscribers.remove(&client_id);
                    if ch.subscribers.is_empty() {
                        d.channels.remove(&channel);
                    }
                }
                reply(&mut writer, &Response::ok()).await?;
            }

            Command::Invite { channel } => {
                let d = daemon.lock().await;
                match d.channels.get(&channel) {
                    None => {
                        reply(
                            &mut writer,
                            &Response::err(format!("Not in channel: {}", channel)),
                        )
                        .await?;
                    }
                    Some(ch) => {
                        let addr = d.endpoint.addr();
                        let ticket = Ticket {
                            topic: ch.topic_id,
                            peers: vec![addr],
                            channel: channel.clone(),
                        };
                        let encoded = data_encoding::BASE32_NOPAD
                            .encode(&postcard::to_stdvec(&ticket).unwrap());
                        let mut resp = Response::ok();
                        resp.ticket = Some(encoded.to_ascii_lowercase());
                        reply(&mut writer, &resp).await?;
                    }
                }
            }

            Command::Accept {
                ticket,
                channel,
                client_id,
            } => {
                let decoded = data_encoding::BASE32_NOPAD
                    .decode(ticket.to_ascii_uppercase().as_bytes());
                match decoded {
                    Ok(bytes) => match postcard::from_bytes::<Ticket>(&bytes) {
                        Ok(ticket_data) => {
                            // Use channel name from ticket if user didn't specify one
                            let ch_name = if channel.is_empty() && !ticket_data.channel.is_empty() {
                                ticket_data.channel.clone()
                            } else if channel.is_empty() {
                                "default".to_string()
                            } else {
                                channel.clone()
                            };
                            let mut d = daemon.lock().await;
                            let mut resp = d
                                .accept_ticket(&ch_name, &ticket_data, &client_id)
                                .await;
                            // Return the resolved channel name so CLI can display it
                            if resp.ok {
                                resp.channel = Some(ch_name);
                            }
                            reply(&mut writer, &resp).await?;
                        }
                        Err(e) => {
                            reply(&mut writer, &Response::err(format!("Bad ticket: {}", e)))
                                .await?;
                        }
                    },
                    Err(e) => {
                        reply(&mut writer, &Response::err(format!("Bad ticket: {}", e)))
                            .await?;
                    }
                }
            }

            Command::Peers { channel } => {
                let d = daemon.lock().await;
                match d.channels.get(&channel) {
                    None => {
                        reply(
                            &mut writer,
                            &Response::err(format!("Not in channel: {}", channel)),
                        )
                        .await?;
                    }
                    Some(ch) => {
                        let peers: Vec<PeerInfo> = ch
                            .peers
                            .iter()
                            .map(|(short, full)| PeerInfo {
                                id: full.to_string(),
                                short: short.clone(),
                            })
                            .collect();
                        let mut resp = Response::ok();
                        resp.peers = Some(peers);
                        reply(&mut writer, &resp).await?;
                    }
                }
            }

            Command::Status => {
                let d = daemon.lock().await;
                let mut channels = HashMap::new();
                for (name, ch) in &d.channels {
                    let buffered: usize = ch
                        .subscribers
                        .keys()
                        .map(|s| d.store.unread_count(name, s))
                        .sum();
                    channels.insert(
                        name.clone(),
                        ChannelInfo {
                            peers: ch.peers.len(),
                            subscribers: ch.subscribers.len(),
                            buffered,
                            total: d.store.total_count(name),
                        },
                    );
                }
                let mut resp = Response::ok();
                resp.channels = Some(channels);
                resp.daemon_id = Some(d.id.clone());
                resp.id = Some(d.endpoint.id().to_string());
                reply(&mut writer, &resp).await?;
            }

            Command::Address => {
                let d = daemon.lock().await;
                let mut resp = Response::ok();
                resp.id = Some(d.endpoint.id().to_string());
                reply(&mut writer, &resp).await?;
            }

            Command::Ping => {
                reply(&mut writer, &Response::ok()).await?;
            }

            Command::Stop => {
                reply(&mut writer, &Response::ok()).await?;
                shutdown().await;
            }
        }
    }
    Ok(())
}

async fn reply(w: &mut tokio::net::unix::OwnedWriteHalf, resp: &Response) -> Result<()> {
    let mut data = serde_json::to_vec(resp)?;
    data.push(b'\n');
    w.write_all(&data).await?;
    Ok(())
}

// ── Ticket ──────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Ticket {
    topic: TopicId,
    peers: Vec<EndpointAddr>,
    #[serde(default)]
    channel: String,
}

// ── Wire message (over gossip) ──────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WireMessage {
    from: String,
    data: String,
    ts: String,
    /// Inline file attachment (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<WireFile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WireFile {
    name: String,
    size: usize,
    data: String, // base64
}

// ── Daemon methods ──────────────────────────────────────────────────

impl Daemon {
    async fn join_channel(&mut self, name: &str, client_id: &str) -> Response {
        let topic_id = derive_topic(name);

        if let Some(ch) = self.channels.get_mut(name) {
            ch.subscribers
                .entry(client_id.to_string())
                .or_insert_with(|| Subscriber {
                    notify: Arc::new(Notify::new()),
                });
            let mut resp = Response::ok();
            resp.channel = Some(name.to_string());
            return resp;
        }

        info!("Joining channel \"{}\" topic={}", name, hex::encode(topic_id.as_bytes()));

        // Subscribe to gossip (no bootstrap peers — they join via invite)
        let gossip_topic = match self.gossip.subscribe(topic_id, vec![]).await {
            Ok(t) => t,
            Err(e) => return Response::err(format!("Gossip subscribe failed: {}", e)),
        };

        let (sender, receiver) = gossip_topic.split();

        // Spawn gossip receive loop
        let store = self.store.clone();
        let chan_name = name.to_string();
        let channel_notify = Arc::new(Notify::new());
        let cn = channel_notify.clone();

        tokio::spawn(async move {
            gossip_receive_loop(receiver, store, chan_name, cn).await;
        });

        let mut subscribers = HashMap::new();
        subscribers.insert(
            client_id.to_string(),
            Subscriber {
                notify: channel_notify,
            },
        );

        self.channels.insert(
            name.to_string(),
            Channel {
                name: name.to_string(),
                topic_id,
                gossip_sender: Some(sender),
                peers: HashMap::new(),
                subscribers,
            },
        );

        let mut resp = Response::ok();
        resp.channel = Some(name.to_string());
        resp
    }

    async fn send_message(
        &mut self,
        channel: &str,
        message: &str,
        sender_id: &str,
        file: Option<&crate::protocol::FileAttachment>,
    ) -> Response {
        let ch = match self.channels.get_mut(channel) {
            Some(ch) => ch,
            None => return Response::err(format!("Not in channel: {}", channel)),
        };

        let ts = now_iso();
        let wire_file = file.map(|f| WireFile {
            name: f.name.clone(),
            size: f.size,
            data: f.data.clone(),
        });
        let wire = serde_json::to_vec(&WireMessage {
            from: sender_id.to_string(),
            data: message.to_string(),
            ts: ts.clone(),
            file: wire_file,
        })
        .unwrap();

        let mut remote_count = 0;
        if let Some(ref sender) = ch.gossip_sender {
            if sender.broadcast(Bytes::from(wire)).await.is_ok() {
                remote_count = ch.peers.len();
            }
        }

        // Persist — save file to ~/.talkd/files/<channel>/ (same for sender and receiver)
        if let Some(f) = file {
            if let Err(e) = self.store.push_message_with_inline_file(
                channel, sender_id, message, ts, remote_count > 0,
                &f.name, &f.data,
            ) {
                warn!("Store error: {}", e);
            }
        } else if let Err(e) = self.store.push_message(channel, sender_id, message, ts, remote_count > 0) {
            warn!("Store error: {}", e);
        }

        let mut local_count = 0;
        for (id, sub) in &ch.subscribers {
            if id != sender_id {
                sub.notify.notify_one();
                local_count += 1;
            }
        }

        let mut resp = Response::ok();
        resp.delivered = Some(remote_count + local_count);
        resp
    }

    async fn send_dm(
        &mut self,
        target_hex: &str,
        message: &str,
        sender_id: &str,
        file: Option<&crate::protocol::FileAttachment>,
    ) -> Response {
        let target_id = match EndpointId::from_str(target_hex) {
            Ok(id) => id,
            Err(e) => return Response::err(format!("Invalid NodeId: {}", e)),
        };

        // ECDH-secured DM topic — only these two nodes can derive it
        let dm_topic = dm_topic_id(&self.secret_key, &target_id);
        let dm_channel = format!("dm:{}", target_id.fmt_short());

        // Ensure we have a gossip subscription for this DM pair
        if !self.channels.contains_key(&dm_channel) {
            let gossip_topic = match self
                .gossip
                .subscribe_and_join(dm_topic, vec![target_id])
                .await
            {
                Ok(t) => t,
                Err(e) => return Response::err(format!("DM connect failed: {}", e)),
            };

            let (sender, receiver) = gossip_topic.split();
            let store = self.store.clone();
            let cn_name = dm_channel.clone();
            let notify = Arc::new(Notify::new());
            let n = notify.clone();

            tokio::spawn(async move {
                gossip_receive_loop(receiver, store, cn_name, n).await;
            });

            let mut subscribers = HashMap::new();
            subscribers.insert(
                sender_id.to_string(),
                Subscriber { notify },
            );

            let mut peers = HashMap::new();
            peers.insert(target_id.fmt_short().to_string(), target_id);

            self.channels.insert(
                dm_channel.clone(),
                Channel {
                    name: dm_channel.clone(),
                    topic_id: dm_topic,
                    gossip_sender: Some(sender),
                    peers,
                    subscribers,
                },
            );
        }

        // Now send
        self.send_message(&dm_channel, message, sender_id, file).await
    }

    async fn accept_ticket(
        &mut self,
        channel: &str,
        ticket: &Ticket,
        client_id: &str,
    ) -> Response {
        let topic_id = ticket.topic;
        let my_id = self.endpoint.id();
        let bootstrap: Vec<EndpointId> = ticket
            .peers
            .iter()
            .map(|p| p.id)
            .filter(|id| *id != my_id)
            .collect();

        if bootstrap.is_empty() {
            return Response::err("Ticket contains no reachable peers (only yourself)".to_string());
        }

        let gossip_topic = match self
            .gossip
            .subscribe_and_join(topic_id, bootstrap.clone())
            .await
        {
            Ok(t) => t,
            Err(e) => return Response::err(format!("Failed to join via ticket: {}", e)),
        };

        let (sender, receiver) = gossip_topic.split();
        let store = self.store.clone();
        let chan_name = channel.to_string();
        let channel_notify = Arc::new(Notify::new());
        let cn = channel_notify.clone();

        tokio::spawn(async move {
            gossip_receive_loop(receiver, store, chan_name, cn).await;
        });

        let mut subscribers = HashMap::new();
        subscribers.insert(
            client_id.to_string(),
            Subscriber {
                notify: channel_notify,
            },
        );

        let mut peers = HashMap::new();
        for id in &bootstrap {
            peers.insert(id.fmt_short().to_string(), *id);
        }

        self.channels.insert(
            channel.to_string(),
            Channel {
                name: channel.to_string(),
                topic_id,
                gossip_sender: Some(sender),
                peers,
                subscribers,
            },
        );

        let mut resp = Response::ok();
        resp.channel = Some(channel.to_string());
        resp
    }
}

// ── DM topic derivation (ECDH-secured) ─────────────────────────────

/// Derive a DM topic that only the two parties can compute.
/// Uses X25519 ECDH: shared_secret = DH(my_private, their_public)
/// Eve knowing both public keys cannot derive this.
fn dm_topic_id(my_secret: &iroh::SecretKey, their_public: &EndpointId) -> TopicId {
    use sha2::{Digest, Sha256};
    use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

    // Convert ed25519 secret → X25519 secret (clamped)
    let ed_secret_bytes = my_secret.to_bytes();
    let x_secret = StaticSecret::from(ed_secret_bytes);

    // Convert ed25519 public → X25519 public (via Montgomery form)
    let ed_public_bytes: [u8; 32] = *their_public.as_bytes();
    let ed_point = curve25519_dalek::edwards::CompressedEdwardsY(ed_public_bytes);
    let x_public_bytes: [u8; 32] = match ed_point.decompress() {
        Some(point) => {
            let montgomery = point.to_montgomery();
            montgomery.to_bytes()
        }
        None => {
            // Fallback: hash-based (shouldn't happen with valid keys)
            let mut h = Sha256::new();
            h.update(b"talkd:dm:fallback:");
            h.update(ed_public_bytes);
            let hash: [u8; 32] = h.finalize().into();
            hash
        }
    };
    let x_public = X25519Public::from(x_public_bytes);

    // ECDH shared secret — same result regardless of who computes it
    let shared_secret = x_secret.diffie_hellman(&x_public);

    let mut hasher = Sha256::new();
    hasher.update(b"talkd:dm:");
    hasher.update(shared_secret.as_bytes());
    TopicId::from_bytes(hasher.finalize().into())
}

// ── Gossip receive loop ─────────────────────────────────────────────

async fn gossip_receive_loop(
    mut receiver: iroh_gossip::api::GossipReceiver,
    store: Store,
    channel: String,
    notify: Arc<Notify>,
) {
    use iroh_gossip::api::Event;
    use n0_future::StreamExt;

    info!("Gossip recv loop started for \"{}\"", channel);

    while let Some(event) = receiver.next().await {
        match event {
            Ok(Event::Received(msg)) => {
                if let Ok(wire) = serde_json::from_slice::<WireMessage>(&msg.content) {
                    debug!("Gossip recv on \"{}\": from={}", channel, wire.from);
                    let result = if let Some(ref f) = wire.file {
                        store.push_message_with_inline_file(
                            &channel, &wire.from, &wire.data, wire.ts, true,
                            &f.name, &f.data,
                        )
                    } else {
                        store.push_message(&channel, &wire.from, &wire.data, wire.ts, true)
                    };
                    if let Err(e) = result {
                        warn!("Store error: {}", e);
                    }
                    notify.notify_waiters();
                }
            }
            Ok(_) => {}
            Err(e) => {
                warn!("Gossip error on \"{}\": {}", channel, e);
                break;
            }
        }
    }
    info!("Gossip recv loop ended for \"{}\"", channel);
}

// ── Shutdown ────────────────────────────────────────────────────────

async fn shutdown() {
    let _ = std::fs::remove_file(socket_path());
    let _ = std::fs::remove_file(pid_path());
    info!("Daemon shutting down");
    std::process::exit(0);
}
