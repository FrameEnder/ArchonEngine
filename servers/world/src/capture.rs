//! Optional packet capture for debugging, enabled with `packet_capture` in the world config.
//!
//! Every packet segment between the server and a game client is written as one JSON object per line to
//! `<packet_capture_dir>/zone-<unix time>-<connection id>.jsonl`:
//!
//! ```json
//! {"t":1760300000123,"dir":"out","src":123,"dst":123,"kind":"Ipc","name":"UpdateQuest","size":40,"hex":"...","decoded":"UpdateQuest { ... }"}
//! ```
//!
//! * `dir` is `out` (server to client) or `in` (client to server.)
//! * `name` is Kawari's name for the IPC data (or the segment type for non-IPC segments.)
//! * `hex` is the IPC segment as written on the wire, including its 16-byte IPC header.
//! * `decoded` is how Kawari understood it.

use std::{
    fs::File,
    io::{BufWriter, Cursor, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use binrw::BinWrite;
use kawari::{
    config::WorldConfig,
    ipc::zone::{ClientZoneIpcSegment, ServerZoneIpcSegment},
    packet::{PacketSegment, SegmentData},
};
use serde_json::json;

/// Longest `decoded` text kept per packet, so huge lists don't bloat the capture.
const MAX_DECODED_LENGTH: usize = 8192;

#[derive(Default)]
pub struct PacketCapture {
    writer: Option<BufWriter<File>>,
}

impl std::fmt::Debug for PacketCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketCapture")
            .field("enabled", &self.writer.is_some())
            .finish()
    }
}

impl PacketCapture {
    /// Starts a capture for a new connection, if enabled in the config.
    pub fn new(config: &WorldConfig, connection_id: impl std::fmt::Debug) -> Self {
        if !config.packet_capture {
            return Self::default();
        }

        let dir = PathBuf::from(&config.packet_capture_dir);
        if let Err(err) = std::fs::create_dir_all(&dir) {
            tracing::warn!("Can't create packet capture folder {dir:?}: {err}");
            return Self::default();
        }

        let id = format!("{connection_id:?}")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
        let path = dir.join(format!("zone-{}-{id}.jsonl", unix_millis() / 1000));
        match File::create(&path) {
            Ok(file) => {
                tracing::info!("Capturing packets to {path:?}");
                Self {
                    writer: Some(BufWriter::new(file)),
                }
            }
            Err(err) => {
                tracing::warn!("Can't create packet capture {path:?}: {err}");
                Self::default()
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.writer.is_some()
    }

    /// Records a segment the server sent.
    pub fn record_outgoing(&mut self, segment: &PacketSegment<ServerZoneIpcSegment>) {
        if !self.is_enabled() {
            return;
        }

        let ipc = match &segment.data {
            SegmentData::Ipc(ipc) => Some(encode(ipc)),
            _ => None,
        };
        self.record("out", segment, ipc);
    }

    /// Records a segment the client sent.
    pub fn record_incoming(&mut self, segment: &PacketSegment<ClientZoneIpcSegment>) {
        if !self.is_enabled() {
            return;
        }

        let ipc = match &segment.data {
            SegmentData::Ipc(ipc) => Some(encode(ipc)),
            _ => None,
        };
        self.record("in", segment, ipc);
    }

    /// Adds a free-form note to the capture, e.g. what a script did.
    pub fn note(&mut self, text: &str) {
        let Some(writer) = &mut self.writer else {
            return;
        };

        let line = json!({ "t": unix_millis(), "dir": "note", "text": text });
        let _ = writeln!(writer, "{line}");
        let _ = writer.flush();
    }

    fn record<T: kawari::packet::ReadWriteIpcSegment>(
        &mut self,
        direction: &str,
        segment: &PacketSegment<T>,
        ipc: Option<(String, Vec<u8>)>,
    ) {
        let Some(writer) = &mut self.writer else {
            return;
        };

        let (name, hex, decoded) = match ipc {
            Some((decoded, bytes)) => {
                let name = decoded
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or_default()
                    .to_string();
                (name, to_hex(&bytes), decoded)
            }
            None => {
                let decoded = format!("{:?}", segment.data);
                (
                    format!("{:?}", segment.segment_type),
                    String::new(),
                    decoded,
                )
            }
        };

        let mut decoded = decoded;
        if decoded.len() > MAX_DECODED_LENGTH {
            let mut cut = MAX_DECODED_LENGTH;
            while !decoded.is_char_boundary(cut) {
                cut -= 1;
            }
            decoded.truncate(cut);
            decoded.push('…');
        }

        let line = json!({
            "t": unix_millis(),
            "dir": direction,
            "src": segment.source_actor.0,
            "dst": segment.target_actor.0,
            "kind": format!("{:?}", segment.segment_type),
            "name": name,
            "size": hex.len() / 2,
            "hex": hex,
            "decoded": decoded,
        });
        let _ = writeln!(writer, "{line}");
        // Flush right away, so the capture is complete even if the server is killed.
        let _ = writer.flush();
    }
}

/// Returns how Kawari decoded an IPC segment's data, and its bytes as written on the wire.
fn encode<T>(ipc: &T) -> (String, Vec<u8>)
where
    T: std::fmt::Debug + for<'a> BinWrite<Args<'a> = ()>,
    T: IpcData,
{
    let mut cursor = Cursor::new(Vec::new());
    let bytes = match ipc.write_le(&mut cursor) {
        Ok(()) => cursor.into_inner(),
        Err(_) => Vec::new(),
    };
    (ipc.data_debug(), bytes)
}

/// The decoded data of an IPC segment, without its header.
trait IpcData {
    fn data_debug(&self) -> String;
}

impl IpcData for ServerZoneIpcSegment {
    fn data_debug(&self) -> String {
        format!("{:?}", self.data)
    }
}

impl IpcData for ClientZoneIpcSegment {
    fn data_debug(&self) -> String {
        format!("{:?}", self.data)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kawari::ipc::zone::{ActiveQuest, ServerZoneIpcData};

    #[test]
    fn captures_outgoing_ipc() {
        let dir = std::env::temp_dir().join(format!("kawari-capture-test-{}", unix_millis()));
        let config = WorldConfig {
            packet_capture: true,
            packet_capture_dir: dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut capture = PacketCapture::new(&config, "ClientId(7)");
        assert!(capture.is_enabled());

        let segment = PacketSegment {
            segment_type: kawari::packet::SegmentType::Ipc,
            data: SegmentData::Ipc(ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateQuest {
                index: 2,
                quest: ActiveQuest {
                    id: 124,
                    sequence: 1,
                    bitflags: [0x01, 0x10, 0x10, 0, 0, 0],
                    ..Default::default()
                },
            })),
            ..Default::default()
        };
        capture.record_outgoing(&segment);
        capture.note("hello");
        drop(capture);

        let file = std::fs::read_dir(&dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let text = std::fs::read_to_string(file).unwrap();
        let mut lines = text.lines();

        let packet: serde_json::Value = serde_json::from_str(lines.next().unwrap()).unwrap();
        assert_eq!(packet["dir"], "out");
        assert_eq!(packet["name"], "UpdateQuest");
        assert!(packet["decoded"].as_str().unwrap().contains("id: 124"));
        // 16 byte IPC header + 24 bytes of UpdateQuest
        assert_eq!(packet["size"], 40);
        // The quest entry starts 4 bytes into the data: 7c 00 (id 124), then the sequence.
        let hex = packet["hex"].as_str().unwrap();
        assert_eq!(&hex[32 + 8..32 + 14], "7c0001");

        let note: serde_json::Value = serde_json::from_str(lines.next().unwrap()).unwrap();
        assert_eq!(note["text"], "hello");

        let _ = std::fs::remove_dir_all(dir);
    }
}
