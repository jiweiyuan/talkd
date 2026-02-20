mod client;
mod crypto;
mod daemon;
mod protocol;
mod store;

use clap::{Parser, Subcommand};
use protocol::{Command, Response};
use std::io::Read;

#[derive(Parser)]
#[command(
    name = "talkd",
    version = "0.3.0",
    about = "P2P communication for AI agents. No server. No setup. Just talk."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize identity (generates persistent keypair)
    Init,

    /// Show your ID
    Id,

    /// Add a contact
    Add {
        /// Name for this contact
        name: String,
        /// ID (64 hex chars, from `talkd id`)
        address: String,
        /// Note about this contact (role, skills, description)
        #[arg(short, long)]
        note: Option<String>,
    },

    /// List all contacts
    Contacts,

    /// Create a new channel (prints invite ticket)
    Create {
        /// Channel name
        channel: String,
    },

    /// Join a channel via invite ticket
    Join {
        /// Invite ticket from `talkd create` or `talkd invite`
        ticket: String,
    },

    /// Send a message to a channel
    Send {
        channel: String,
        /// Message text (reads stdin if omitted)
        message: Option<String>,
        /// Attach a file
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Send a direct message (by ID or contact name)
    Dm {
        /// Target ID or contact name
        target: String,
        /// Message text (reads stdin if omitted)
        message: Option<String>,
        /// Attach a file
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Read pending messages from a channel
    Read {
        channel: String,
        /// Block until a message arrives
        #[arg(short, long)]
        wait: bool,
        /// Timeout in seconds (for --wait)
        #[arg(short, long)]
        timeout: Option<u64>,
    },

    /// Stream messages as they arrive
    Listen { channel: String },

    /// Generate an invite ticket for a channel
    Invite { channel: String },



    /// List peers in a channel (shows full IDs for adding contacts)
    Peers { channel: String },

    /// Show active channels, peers, and status
    Status,

    /// Leave a channel
    Leave { channel: String },

    /// Stop the daemon
    Stop,

    /// Internal: run as daemon
    #[command(name = "__daemon", hide = true)]
    Daemon,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match cli.command {
        Commands::Daemon => daemon::run_daemon().await,
        Commands::Init => cmd_init(json),
        Commands::Id => cmd_id(json).await,
        Commands::Add { name, address, note } => cmd_add(name, address, note, json),
        Commands::Contacts => cmd_contacts(json),
        Commands::Create { channel } => cmd_create(channel, json).await,
        Commands::Join { ticket } => cmd_join(ticket, json).await,
        Commands::Send { channel, message, file } => cmd_send(channel, message, file, json).await,
        Commands::Dm { target, message, file } => cmd_dm(target, message, file, json).await,
        Commands::Read {
            channel,
            wait,
            timeout,
        } => cmd_read(channel, wait, timeout, json).await,
        Commands::Listen { channel } => cmd_listen(channel, json).await,
        Commands::Invite { channel } => cmd_invite(channel, json).await,
        Commands::Peers { channel } => cmd_peers(channel, json).await,
        Commands::Status => cmd_status(json).await,
        Commands::Leave { channel } => cmd_leave(channel, json).await,
        Commands::Stop => cmd_stop(json).await,
    };

    if let Err(e) = result {
        if json {
            println!(
                "{}",
                serde_json::to_string(&Response::err(e.to_string())).unwrap()
            );
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}

// ── Local commands (no daemon needed) ───────────────────────────────

fn cmd_init(json: bool) -> anyhow::Result<()> {
    let dir = daemon::talkd_dir();
    let secret = crypto::load_or_create_identity(&dir)?;
    let public = secret.public();
    if json {
        let mut resp = Response::ok();
        resp.id = Some(public.to_string());
        println!("{}", serde_json::to_string(&resp)?);
    } else {
        println!("Identity ready");
        println!("ID: {}", public);
        println!("Stored:  {}/.talkd/identity", dirs::home_dir().unwrap_or_default().display());
    }
    Ok(())
}

fn cmd_add(name: String, address: String, note: Option<String>, json: bool) -> anyhow::Result<()> {
    let dir = daemon::talkd_dir();
    let mut contacts = crypto::load_contacts(&dir);

    let pubkey: iroh::PublicKey = address
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid ID. Expected 64 hex chars (from `talkd id`)."))?;

    let contact = crypto::Contact {
        id: pubkey,
        note: note.clone().unwrap_or_default(),
    };
    contacts.insert(name.clone(), contact);
    crypto::save_contacts(&dir, &contacts)?;

    if json {
        let mut resp = Response::ok();
        resp.id = Some(pubkey.to_string());
        println!("{}", serde_json::to_string(&resp)?);
    } else {
        println!("Added contact \"{}\"", name);
        if let Some(n) = &note {
            println!("  note: {}", n);
        }
        println!("\nNow you can: talkd dm {} \"hello\"", name);
    }
    Ok(())
}

fn cmd_contacts(json: bool) -> anyhow::Result<()> {
    let dir = daemon::talkd_dir();
    let contacts = crypto::load_contacts(&dir);
    if json {
        println!("{}", serde_json::to_string(&contacts)?);
    } else if contacts.is_empty() {
        println!("No contacts.");
        println!("\nAdd one:");
        println!("  talkd add <name> <id> --note \"description\"");
    } else {
        for (name, c) in &contacts {
            let short = c.id.fmt_short();
            if c.note.is_empty() {
                println!("  {} → {}", name, short);
            } else {
                println!("  {} → {} ({})", name, short, c.note);
            }
        }
    }
    Ok(())
}

// ── Daemon commands ─────────────────────────────────────────────────

fn read_message_text(message: Option<String>) -> anyhow::Result<String> {
    match message {
        Some(m) => Ok(m),
        None => {
            if atty::is(atty::Stream::Stdin) {
                anyhow::bail!("No message. Pass as argument or pipe to stdin.");
            }
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf.trim_end().to_string())
        }
    }
}

async fn cmd_create(channel: String, json: bool) -> anyhow::Result<()> {
    // 1. Join the channel locally
    let join_resp = client::request(
        Command::Join {
            channel: channel.clone(),
            client_id: crypto::client_id(),
        },
        30_000,
    )
    .await?;
    if !join_resp.ok {
        if json {
            println!("{}", serde_json::to_string(&join_resp)?);
        } else {
            eprintln!("Error: {}", join_resp.error.unwrap_or_default());
        }
        std::process::exit(1);
    }

    // 2. Generate invite ticket
    let invite_resp = client::request(Command::Invite { channel: channel.clone() }, 10_000).await?;
    if json {
        println!("{}", serde_json::to_string(&invite_resp)?);
    } else if invite_resp.ok {
        let ticket = invite_resp.ticket.unwrap_or_default();
        println!("Created channel \"{}\"", channel);
        println!("Invite ticket:\n{}", ticket);
    } else {
        eprintln!("Error: {}", invite_resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_join(ticket: String, json: bool) -> anyhow::Result<()> {
    let resp = client::request(
        Command::Accept {
            ticket,
            channel: String::new(),
            client_id: crypto::client_id(),
        },
        30_000,
    )
    .await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        let ch = resp.channel.unwrap_or_else(|| "default".to_string());
        println!("Joined channel \"{}\"", ch);
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_send(
    channel: String,
    message: Option<String>,
    file: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let attachment = load_file_attachment(file.as_deref())?;
    let msg = if message.is_some() || attachment.is_some() {
        message.unwrap_or_default()
    } else {
        read_message_text(None)?
    };
    let resp = client::request(
        Command::Send {
            channel: channel.clone(),
            message: msg,
            client_id: crypto::client_id(),
            file: attachment,
        },
        30_000,
    )
    .await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        let n = resp.delivered.unwrap_or(0);
        println!(
            "Sent (delivered to {} recipient{})",
            n,
            if n != 1 { "s" } else { "" }
        );
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_dm(
    target: String,
    message: Option<String>,
    file: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let dir = daemon::talkd_dir();
    let pubkey = crypto::resolve_target(&dir, &target)
        .ok_or_else(|| anyhow::anyhow!("Unknown target '{}'. Use ID or contact name.", target))?;
    let attachment = load_file_attachment(file.as_deref())?;
    let msg = if message.is_some() || attachment.is_some() {
        message.unwrap_or_default()
    } else {
        read_message_text(None)?
    };
    let resp = client::request(
        Command::Dm {
            target: pubkey.to_string(),
            message: msg,
            client_id: crypto::client_id(),
            file: attachment,
        },
        30_000,
    )
    .await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        let n = resp.delivered.unwrap_or(0);
        println!("DM sent (delivered to {})", n);
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

/// Load a file from disk and encode as a FileAttachment.
fn load_file_attachment(
    path: Option<&str>,
) -> anyhow::Result<Option<protocol::FileAttachment>> {
    let path = match path {
        Some(p) => p,
        None => return Ok(None),
    };
    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", path);
    }
    let bytes = std::fs::read(file_path)?;
    if bytes.len() > protocol::MAX_INLINE_FILE_SIZE {
        anyhow::bail!(
            "File too large: {} ({} bytes, max {})",
            path,
            bytes.len(),
            protocol::MAX_INLINE_FILE_SIZE
        );
    }
    let name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let data = data_encoding::BASE64.encode(&bytes);
    Ok(Some(protocol::FileAttachment {
        name,
        size: bytes.len(),
        data,
    }))
}

async fn cmd_read(
    channel: String,
    wait: bool,
    timeout: Option<u64>,
    json: bool,
) -> anyhow::Result<()> {
    let ipc_timeout = if wait {
        match timeout {
            Some(t) => (t + 5) * 1000,
            None => 0,
        }
    } else {
        10_000
    };
    let resp = client::request(
        Command::Read {
            channel: channel.clone(),
            client_id: crypto::client_id(),
            wait,
            timeout,
        },
        ipc_timeout,
    )
    .await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        let msgs = resp.messages.unwrap_or_default();
        if msgs.is_empty() {
            println!("No new messages");
        } else {
            for msg in &msgs {
                print_message(msg);
            }
        }
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_listen(channel: String, json: bool) -> anyhow::Result<()> {
    loop {
        let resp = client::request(
            Command::Read {
                channel: channel.clone(),
                client_id: crypto::client_id(),
                wait: true,
                timeout: None,
            },
            0,
        )
        .await;
        match resp {
            Ok(r) if r.ok => {
                for msg in r.messages.unwrap_or_default() {
                    if json {
                        println!("{}", serde_json::to_string(&msg).unwrap_or_default());
                    } else {
                        print_message(&msg);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}. Reconnecting...", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            _ => {}
        }
    }
}

async fn cmd_invite(channel: String, json: bool) -> anyhow::Result<()> {
    let resp = client::request(Command::Invite { channel }, 10_000).await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        println!("{}", resp.ticket.unwrap_or_default());
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}


async fn cmd_id(json: bool) -> anyhow::Result<()> {
    // Try daemon first (it knows the live ID)
    match client::request(Command::Address, 5_000).await {
        Ok(resp) if resp.ok => {
            if json {
                println!("{}", serde_json::to_string(&resp)?);
            } else {
                println!("{}", resp.id.unwrap_or_default());
            }
        }
        _ => {
            // Fallback: read from identity file
            let dir = daemon::talkd_dir();
            let secret = crypto::load_or_create_identity(&dir)?;
            let public = secret.public();
            if json {
                let mut resp = Response::ok();
                resp.id = Some(public.to_string());
                println!("{}", serde_json::to_string(&resp)?);
            } else {
                println!("{}", public);
            }
        }
    }
    Ok(())
}

async fn cmd_peers(channel: String, json: bool) -> anyhow::Result<()> {
    let resp = client::request(
        Command::Peers {
            channel: channel.clone(),
        },
        10_000,
    )
    .await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        let peers = resp.peers.unwrap_or_default();
        if peers.is_empty() {
            println!("No peers in channel \"{}\"", channel);
        } else {
            for p in &peers {
                println!("  {} {}", p.short, p.id);
            }
        }
        println!("\nTo DM a peer:");
        println!("  talkd add <name> <id>");
        println!("  talkd dm <name> \"hello\"");
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_status(json: bool) -> anyhow::Result<()> {
    let resp = client::request(Command::Status, 10_000).await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        println!("Daemon ID: {}", resp.daemon_id.unwrap_or_default());
        println!("ID:   {}", resp.id.unwrap_or_default());
        let channels = resp.channels.unwrap_or_default();
        if channels.is_empty() {
            println!("No active channels");
        } else {
            for (name, info) in &channels {
                println!(
                    "  #{} — {} peer(s), {} subscriber(s), {} buffered, {} total",
                    name, info.peers, info.subscribers, info.buffered, info.total
                );
            }
        }
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_leave(channel: String, json: bool) -> anyhow::Result<()> {
    let resp = client::request(
        Command::Leave {
            channel: channel.clone(),
            client_id: crypto::client_id(),
        },
        10_000,
    )
    .await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else if resp.ok {
        println!("Left channel \"{}\"", channel);
    } else {
        eprintln!("Error: {}", resp.error.unwrap_or_default());
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_stop(json: bool) -> anyhow::Result<()> {
    match client::request(Command::Stop, 5_000).await {
        Ok(resp) => {
            if json {
                println!("{}", serde_json::to_string(&resp)?);
            } else {
                println!("Daemon stopped");
            }
        }
        Err(_) => {
            if json {
                println!("{}", serde_json::to_string(&Response::ok())?);
            } else {
                println!("Daemon is not running");
            }
        }
    }
    Ok(())
}

fn print_message(msg: &protocol::Message) {
    // Parse ISO 8601 → local HH:MM:SS for display
    let time_str = chrono::DateTime::parse_from_rfc3339(&msg.ts)
        .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|_| msg.ts.clone());

    if let Some(ref f) = msg.file {
        let size = human_size(f.size);
        if msg.data.is_empty() {
            println!("[{}] {}: 📎 {} ({}) → {}", time_str, msg.from, f.name, size, f.path);
        } else {
            println!("[{}] {}: {} 📎 {} ({}) → {}", time_str, msg.from, msg.data, f.name, size, f.path);
        }
    } else {
        println!("[{}] {}: {}", time_str, msg.from, msg.data);
    }
}

fn human_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
