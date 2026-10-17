//! Framing pesan P2P ZenaChain, mengikuti format Bitcoin:
//! [magic: 4][command: 12][length: 4 LE][checksum: 4][payload: length]
//!
//! Checksum memakai double-SHA3-256 (analog double-SHA256 Bitcoin),
//! ambil 4 byte pertama.

use crate::blockchain::network::NETWORK_MAGIC;
use crate::utils::sha3_256_hex;
use std::io::{self, Read, Write};

pub const COMMAND_LEN: usize = 12;
pub const HEADER_LEN: usize = 4 + COMMAND_LEN + 4 + 4;
pub const MAX_PAYLOAD_LEN: usize = 32 * 1024 * 1024;

pub fn checksum(payload: &[u8]) -> [u8; 4] {
    let first = sha3_256_hex(payload);
    let first_bytes = hex::decode(&first).expect("sha3 hex selalu valid");
    let second = sha3_256_hex(&first_bytes);
    let second_bytes = hex::decode(&second).expect("sha3 hex selalu valid");
    let mut out = [0u8; 4];
    out.copy_from_slice(&second_bytes[..4]);
    out
}

pub fn read_message<R: Read>(reader: &mut R) -> io::Result<(String, Vec<u8>)> {
    let mut header = [0u8; HEADER_LEN];
    reader.read_exact(&mut header)?;

    let magic = &header[0..4];
    if magic != NETWORK_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("magic tidak cocok: {:02x?}", magic),
        ));
    }

    let command_bytes = &header[4..4 + COMMAND_LEN];
    let command_end = command_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(COMMAND_LEN);
    let command = String::from_utf8_lossy(&command_bytes[..command_end]).to_string();

    let length = u32::from_le_bytes([
        header[16], header[17], header[18], header[19],
    ]) as usize;

    if length > MAX_PAYLOAD_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("payload terlalu besar: {} byte", length),
        ));
    }

    let expected_checksum = [header[20], header[21], header[22], header[23]];

    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload)?;

    let actual_checksum = checksum(&payload);
    if actual_checksum != expected_checksum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "checksum tidak cocok",
        ));
    }

    Ok((command, payload))
}

pub fn write_message<W: Write>(writer: &mut W, command: &str, payload: &[u8]) -> io::Result<()> {
    if command.len() > COMMAND_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("command terlalu panjang: {}", command),
        ));
    }

    let mut header = [0u8; HEADER_LEN];
    header[0..4].copy_from_slice(&NETWORK_MAGIC);

    let cmd_bytes = command.as_bytes();
    header[4..4 + cmd_bytes.len()].copy_from_slice(cmd_bytes);

    let length = payload.len() as u32;
    header[16..20].copy_from_slice(&length.to_le_bytes());

    let cs = checksum(payload);
    header[20..24].copy_from_slice(&cs);

    writer.write_all(&header)?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_pesan() {
        let mut buf = Vec::new();
        write_message(&mut buf, "ping", b"hello").unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let (cmd, payload) = read_message(&mut cursor).unwrap();
        assert_eq!(cmd, "ping");
        assert_eq!(payload, b"hello");
    }

    #[test]
    fn checksum_konsisten() {
        let a = checksum(b"zenachain");
        let b = checksum(b"zenachain");
        assert_eq!(a, b);
    }

    #[test]
    fn magic_salah_ditolak() {
        let mut buf = Vec::new();
        write_message(&mut buf, "ping", b"").unwrap();
        buf[0] = 0x00;
        let mut cursor = std::io::Cursor::new(buf);
        assert!(read_message(&mut cursor).is_err());
    }

    #[test]
    fn checksum_salah_ditolak() {
        let mut buf = Vec::new();
        write_message(&mut buf, "ping", b"data").unwrap();
        let last = buf.len() - 1;
        buf[last] ^= 0xff;
        let mut cursor = std::io::Cursor::new(buf);
        assert!(read_message(&mut cursor).is_err());
    }
}
