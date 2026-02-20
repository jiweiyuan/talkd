use crate::protocol::{FileRef, Message};
use anyhow::Result;
use data_encoding::BASE64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// On-disk state: one JSON file per channel + files directory.
///
/// Layout:
///   ~/.talkd/
///     channels/
///       room.json
///     files/
///       room/
///         1234567890-report.csv
#[derive(Clone)]
pub struct Store {
    dir: PathBuf,     // ~/.talkd/channels/
    files_dir: PathBuf, // ~/.talkd/files/
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
struct ChannelData {
    messages: Vec<StoredMessage>,
    cursors: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredMessage {
    from: String,
    data: String,
    ts: String,
    #[serde(default)]
    delivered: bool,
    /// File attachment saved to disk
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<StoredFileRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredFileRef {
    name: String,
    size: usize,
    path: String,
}

#[allow(dead_code)]
impl Store {
    pub fn open(base_dir: &std::path::Path) -> Result<Self> {
        let dir = base_dir.join("channels");
        let files_dir = base_dir.join("attachments");
        std::fs::create_dir_all(&dir)?;
        std::fs::create_dir_all(&files_dir)?;
        Ok(Self { dir, files_dir })
    }

    /// Map channel name to storage name:
    ///   "design"     → "room-design"
    ///   "dm:abc123"  → "dm-abc123"
    fn storage_name(channel: &str) -> String {
        if let Some(peer) = channel.strip_prefix("dm:") {
            format!("dm-{}", sanitize_name(peer))
        } else {
            format!("ch-{}", sanitize_name(channel))
        }
    }

    fn channel_path(&self, channel: &str) -> PathBuf {
        self.dir.join(format!("{}.json", Self::storage_name(channel)))
    }

    fn channel_files_dir(&self, channel: &str) -> PathBuf {
        self.files_dir.join(Self::storage_name(channel))
    }

    fn load(&self, channel: &str) -> ChannelData {
        let path = self.channel_path(channel);
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ChannelData::default(),
        }
    }

    fn save(&self, channel: &str, data: &ChannelData) -> Result<()> {
        let path = self.channel_path(channel);
        let json = serde_json::to_string_pretty(data)?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// Save a file attachment to disk. Returns the local path.
    pub fn save_file(
        &self,
        channel: &str,
        filename: &str,
        data_b64: &str,
        ts: String,
    ) -> Result<(String, usize)> {
        let dir = self.channel_files_dir(channel);
        std::fs::create_dir_all(&dir)?;

        let bytes = BASE64.decode(data_b64.as_bytes())?;
        let size = bytes.len();

        // Unique filename: <YYYYMMDDTHHMMSS>-<original_name>
        let safe_name = sanitize_name(filename);
        let safe_ts = chrono::DateTime::parse_from_rfc3339(&ts)
            .map(|dt| dt.format("%Y%m%dT%H%M%S").to_string())
            .unwrap_or_else(|_| ts.replace(':', "").replace('-', ""));
        let dest = dir.join(format!("{}-{}", safe_ts, safe_name));
        std::fs::write(&dest, &bytes)?;

        Ok((dest.to_string_lossy().into_owned(), size))
    }

    /// Append a message (with optional file ref).
    pub fn push_message(
        &self,
        channel: &str,
        from: &str,
        data: &str,
        ts: String,
        delivered: bool,
    ) -> Result<usize> {
        self.push_message_with_file(channel, from, data, ts, delivered, None)
    }

    /// Append a message with optional file attachment.
    fn push_message_with_file(
        &self,
        channel: &str,
        from: &str,
        data: &str,
        ts: String,
        delivered: bool,
        file: Option<StoredFileRef>,
    ) -> Result<usize> {
        let mut cd = self.load(channel);
        cd.messages.push(StoredMessage {
            from: from.into(),
            data: data.into(),
            ts,
            delivered,
            file,
        });
        let idx = cd.messages.len();
        self.save(channel, &cd)?;
        Ok(idx)
    }

    /// Push a message that includes an inline file (base64).
    /// Saves the file to disk and stores a reference.
    pub fn push_message_with_inline_file(
        &self,
        channel: &str,
        from: &str,
        data: &str,
        ts: String,
        delivered: bool,
        filename: &str,
        file_data_b64: &str,
    ) -> Result<usize> {
        let (path, size) = self.save_file(channel, filename, file_data_b64, ts.clone())?;
        let file_ref = StoredFileRef {
            name: filename.to_string(),
            size,
            path,
        };
        self.push_message_with_file(channel, from, data, ts, delivered, Some(file_ref))
    }

    /// Mark a message as delivered.
    pub fn mark_delivered(&self, channel: &str, idx: usize) -> Result<()> {
        let mut cd = self.load(channel);
        if let Some(msg) = cd.messages.get_mut(idx.saturating_sub(1)) {
            msg.delivered = true;
            self.save(channel, &cd)?;
        }
        Ok(())
    }

    /// Read unread messages for a subscriber. Advances cursor.
    pub fn read_messages(&self, channel: &str, subscriber: &str) -> Result<Vec<Message>> {
        let mut cd = self.load(channel);
        let cursor = cd.cursors.get(subscriber).copied().unwrap_or(0);
        let messages: Vec<Message> = cd.messages[cursor..]
            .iter()
            .map(|m| Message {
                from: m.from.clone(),
                data: m.data.clone(),
                ts: m.ts.clone(),
                file: m.file.as_ref().map(|f| FileRef {
                    name: f.name.clone(),
                    size: f.size,
                    path: f.path.clone(),
                }),
            })
            .collect();

        if !messages.is_empty() {
            cd.cursors.insert(subscriber.into(), cd.messages.len());
            self.save(channel, &cd)?;
        }

        Ok(messages)
    }

    /// Check if there are unread messages.
    pub fn has_unread(&self, channel: &str, subscriber: &str) -> bool {
        let cd = self.load(channel);
        let cursor = cd.cursors.get(subscriber).copied().unwrap_or(0);
        cursor < cd.messages.len()
    }

    /// Count of unread messages for a subscriber.
    pub fn unread_count(&self, channel: &str, subscriber: &str) -> usize {
        let cd = self.load(channel);
        let cursor = cd.cursors.get(subscriber).copied().unwrap_or(0);
        cd.messages.len().saturating_sub(cursor)
    }

    /// Total messages in a channel.
    pub fn total_count(&self, channel: &str) -> usize {
        self.load(channel).messages.len()
    }

    /// Get the latest N messages (for preview in status).
    pub fn latest_messages(&self, channel: &str, n: usize) -> Vec<Message> {
        let cd = self.load(channel);
        let start = cd.messages.len().saturating_sub(n);
        cd.messages[start..]
            .iter()
            .map(|m| Message {
                from: m.from.clone(),
                data: m.data.clone(),
                ts: m.ts.clone(),
                file: m.file.as_ref().map(|f| FileRef {
                    name: f.name.clone(),
                    size: f.size,
                    path: f.path.clone(),
                }),
            })
            .collect()
    }

    /// Get undelivered messages (for retry when peers reconnect).
    pub fn get_undelivered(&self, channel: &str) -> Vec<Message> {
        let cd = self.load(channel);
        cd.messages
            .iter()
            .filter(|m| !m.delivered)
            .map(|m| Message {
                from: m.from.clone(),
                data: m.data.clone(),
                ts: m.ts.clone(),
                file: m.file.as_ref().map(|f| FileRef {
                    name: f.name.clone(),
                    size: f.size,
                    path: f.path.clone(),
                }),
            })
            .collect()
    }

    /// Remove channel data file.
    pub fn remove_channel(&self, channel: &str) -> Result<()> {
        let path = self.channel_path(channel);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    /// Cleanup old messages beyond retention limit.
    pub fn cleanup(&self, channel: &str, max_messages: usize) -> Result<usize> {
        let mut cd = self.load(channel);
        if cd.messages.len() <= max_messages {
            return Ok(0);
        }
        let remove = cd.messages.len() - max_messages;
        cd.messages.drain(..remove);
        for cursor in cd.cursors.values_mut() {
            *cursor = cursor.saturating_sub(remove);
        }
        self.save(channel, &cd)?;
        Ok(remove)
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}
