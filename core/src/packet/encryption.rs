use std::io::Cursor;

use binrw::BinResult;

use crate::constants::LOBBY_GAME_VERSION;

use super::{IPC_HEADER_SIZE, ReadWriteIpcSegment, parsing::ConnectionState, remap};

pub fn generate_encryption_key(key: &[u8], phrase: &str) -> [u8; 16] {
    let mut base_key = vec![0x78, 0x56, 0x34, 0x12]; // 0x12345678
    base_key.extend_from_slice(key);
    base_key.extend_from_slice(&LOBBY_GAME_VERSION.to_le_bytes());
    base_key.extend_from_slice(phrase.as_bytes());

    md5::compute(&base_key).0
}

#[binrw::parser(reader, endian)]
pub(crate) fn decrypt<T: ReadWriteIpcSegment>(size: u32, state: &ConnectionState) -> BinResult<T> {
    if let ConnectionState::Lobby { blowfish } = state {
        let size = size - IPC_HEADER_SIZE;

        let mut data = vec![0; size as usize];
        reader.read_exact(&mut data)?;

        blowfish.decrypt(&mut data);

        let mut cursor = Cursor::new(&data);
        T::read_options(&mut cursor, endian, (&size,))
    } else {
        // Each segment is read from its own bytes, so an IPC whose layout is shorter or longer than what the client
        // sent spoils only itself: read from the stream, a wrong size threw off every segment after it in the packet
        // ("no variants matched" on a segment type: Bob's bundle 20261030-202136, the Waking Sands' door never
        // answered and the client waited in its event.) Past the end it reads zeros, and either way it's logged.
        let length = size.saturating_sub(IPC_HEADER_SIZE) as usize;
        let mut data = vec![0; length + READ_SLACK];
        reader.read_exact(&mut data[..length])?;
        if remap::active() {
            // An opcode found while running comes in with its real opcode, but is read as its placeholder.
            remap::rewrite_opcode(&mut data, remap::kind_of::<T::OpCode>(), remap::incoming);
        }

        let mut cursor = Cursor::new(&data);
        let value = T::read_options(&mut cursor, endian, (&size,))?;
        let read = cursor.position() as usize;
        if read != length {
            tracing::warn!(
                "{} (opcode {}) came with {} bytes but its layout reads {}: check its size",
                value.get_name(),
                value.get_opcode(),
                length.saturating_sub(16),
                read.saturating_sub(16)
            );
        }
        Ok(value)
    }
}

/// Zeros after a segment's bytes, for a layout that reads past what the client sent.
const READ_SLACK: usize = 4096;

#[binrw::writer(writer, endian)]
pub(crate) fn encrypt<T: ReadWriteIpcSegment>(
    value: &T,
    size: u32,
    state: &ConnectionState,
) -> BinResult<()> {
    if let ConnectionState::Lobby { blowfish } = state {
        let size = size - IPC_HEADER_SIZE;

        let mut cursor = Cursor::new(Vec::with_capacity(size as usize));
        value.write_options(&mut cursor, endian, ())?;

        let mut buffer = cursor.into_inner();
        buffer.resize(size as usize, 0);

        blowfish.encrypt(&mut buffer);

        writer.write_all(&buffer)?;

        Ok(())
    } else {
        value.write_options(writer, endian, ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_key() {
        let key = generate_encryption_key(&[0x00, 0x00, 0x00, 0x00], "foobar");
        assert_eq!(
            key,
            [
                147, 75, 201, 132, 255, 202, 150, 130, 59, 31, 162, 205, 133, 248, 14, 1
            ]
        );
    }
}
