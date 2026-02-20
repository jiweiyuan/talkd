use crate::protocol::{Command, Response};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

fn talkd_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("TALKD_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".talkd")
}

fn socket_path() -> PathBuf {
    talkd_dir().join("daemon.sock")
}

fn pid_path() -> PathBuf {
    talkd_dir().join("daemon.pid")
}

/// Connect to the daemon's Unix socket.
async fn connect() -> Result<UnixStream> {
    let path = socket_path();
    UnixStream::connect(&path)
        .await
        .with_context(|| format!("Cannot connect to daemon at {}", path.display()))
}

/// Send a command and read one JSON response line.
async fn send_command(stream: &mut UnixStream, cmd: &Command, timeout_ms: u64) -> Result<Response> {
    let mut data = serde_json::to_vec(cmd)?;
    data.push(b'\n');
    stream.write_all(&data).await?;

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    if timeout_ms == 0 {
        // No timeout — block forever
        let line = lines
            .next_line()
            .await?
            .context("Connection closed before response")?;
        Ok(serde_json::from_str(&line)?)
    } else {
        let result = tokio::time::timeout(Duration::from_millis(timeout_ms), lines.next_line()).await;
        match result {
            Ok(Ok(Some(line))) => Ok(serde_json::from_str(&line)?),
            Ok(Ok(None)) => anyhow::bail!("Connection closed before response"),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => anyhow::bail!("Command timed out"),
        }
    }
}

/// Check if a process is running.
fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

/// Ensure the daemon is running, start it if not.
async fn ensure_daemon() -> Result<()> {
    // Try connecting to existing daemon
    if let Ok(mut stream) = connect().await {
        let cmd = Command::Ping;
        if let Ok(resp) = send_command(&mut stream, &cmd, 5000).await {
            if resp.ok {
                return Ok(());
            }
        }
    }

    // Clean stale socket
    let _ = std::fs::remove_file(socket_path());

    // Clean stale PID file
    if let Ok(content) = std::fs::read_to_string(pid_path()) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            if !is_process_running(pid) {
                let _ = std::fs::remove_file(pid_path());
            }
        }
    }

    // Create daemon directory
    let dir = talkd_dir();
    std::fs::create_dir_all(&dir)?;

    // Spawn daemon process
    let exe = std::env::current_exe()?;
    let child = std::process::Command::new(&exe)
        .arg("__daemon")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn daemon: {}", exe.display()))?;

    tracing::debug!("Spawned daemon pid={}", child.id());

    // Poll until daemon is ready
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(200)).await;
        if let Ok(mut stream) = connect().await {
            let cmd = Command::Ping;
            if let Ok(resp) = send_command(&mut stream, &cmd, 5000).await {
                if resp.ok {
                    return Ok(());
                }
            }
        }
    }

    anyhow::bail!("Failed to start talkd daemon. Check ~/.talkd/daemon.log")
}

/// Send a command to the daemon, auto-starting it if needed.
pub async fn request(cmd: Command, timeout_ms: u64) -> Result<Response> {
    ensure_daemon().await?;
    let mut stream = connect().await?;
    send_command(&mut stream, &cmd, timeout_ms).await
}

/// Send a command and stream responses (for `listen` command).
#[allow(dead_code)]
pub async fn request_stream(
    cmd: Command,
    mut on_response: impl FnMut(Response) -> bool,
) -> Result<()> {
    ensure_daemon().await?;
    let mut stream = connect().await?;

    let mut data = serde_json::to_vec(&cmd)?;
    data.push(b'\n');
    stream.write_all(&data).await?;

    let reader = BufReader::new(&mut stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let resp: Response = serde_json::from_str(&line)?;
        if !on_response(resp) {
            break;
        }
    }

    Ok(())
}
