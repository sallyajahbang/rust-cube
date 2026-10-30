//! Pembaca region file Minecraft (.mca).
//! Satu region berisi 32x32 chunk. Format: header 8 KiB, lalu data per sektor 4 KiB.

use std::io::Read;
use std::path::Path;

use flate2::read::{GzDecoder, ZlibDecoder};

pub const SECTOR: usize = 4096;

#[derive(Debug)]
pub enum RegionError {
    Io(std::io::Error),
    TooSmall,
    BadLength,
    Compression(u8),
    External,
    Decompress(std::io::Error),
}

impl From<std::io::Error> for RegionError {
    fn from(e: std::io::Error) -> Self {
        RegionError::Io(e)
    }
}

pub struct Region {
    data: Vec<u8>,
}

impl Region {
    pub fn open(path: &Path) -> Result<Self, RegionError> {
        Self::from_bytes(std::fs::read(path)?)
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, RegionError> {
        if data.len() < 2 * SECTOR {
            return Err(RegionError::TooSmall);
        }
        Ok(Region { data })
    }

    /// Mengambil NBT mentah (sudah di-decompress) untuk chunk lokal (x, z), 0..32.
    /// Mengembalikan None kalau chunk belum pernah di-generate.
    pub fn chunk_nbt(&self, x: usize, z: usize) -> Result<Option<Vec<u8>>, RegionError> {
        if x >= 32 || z >= 32 {
            return Ok(None);
        }
        let idx = 4 * (x + z * 32);
        let loc = &self.data[idx..idx + 4];
        let offset = (((loc[0] as usize) << 16) | ((loc[1] as usize) << 8) | loc[2] as usize) * SECTOR;
        let sectors = loc[3] as usize;
        if offset == 0 || sectors == 0 {
            return Ok(None);
        }

        let end = offset + sectors * SECTOR;
        if end > self.data.len() {
            return Err(RegionError::BadLength);
        }
        let c = &self.data[offset..end];

        // 4 byte panjang (termasuk 1 byte tipe kompresi), lalu tipe, lalu payload.
        let len = u32::from_be_bytes([c[0], c[1], c[2], c[3]]) as usize;
        if len == 0 || 4 + len > c.len() {
            return Err(RegionError::BadLength);
        }
        let kind = c[4];
        if kind & 0x80 != 0 {
            // Chunk disimpan di file .mcc terpisah. Belum didukung.
            return Err(RegionError::External);
        }
        let payload = &c[5..4 + len];

        let mut out = Vec::new();
        match kind {
            1 => {
                GzDecoder::new(payload).read_to_end(&mut out).map_err(RegionError::Decompress)?;
            }
            2 => {
                ZlibDecoder::new(payload).read_to_end(&mut out).map_err(RegionError::Decompress)?;
            }
            3 => out.extend_from_slice(payload),
            other => return Err(RegionError::Compression(other)),
        }
        Ok(Some(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    /// Membuat region berisi satu chunk di (0, 0) dengan payload dan tipe kompresi tertentu.
    fn build(payload: &[u8], kind: u8) -> Vec<u8> {
        let mut data = vec![0u8; 2 * SECTOR];
        // Lokasi chunk (0,0): mulai di sektor 2, panjang 1 sektor.
        data[0..4].copy_from_slice(&[0, 0, 2, 1]);
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&((payload.len() + 1) as u32).to_be_bytes());
        chunk.push(kind);
        chunk.extend_from_slice(payload);
        chunk.resize(SECTOR, 0);
        data.extend_from_slice(&chunk);
        data
    }

    #[test]
    fn reads_uncompressed_chunk() {
        let region = Region::from_bytes(build(b"hello", 3)).unwrap();
        assert_eq!(region.chunk_nbt(0, 0).unwrap(), Some(b"hello".to_vec()));
    }

    #[test]
    fn reads_zlib_chunk() {
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(b"compressed data").unwrap();
        let compressed = enc.finish().unwrap();
        let region = Region::from_bytes(build(&compressed, 2)).unwrap();
        assert_eq!(region.chunk_nbt(0, 0).unwrap(), Some(b"compressed data".to_vec()));
    }

    #[test]
    fn missing_chunk_is_none() {
        let region = Region::from_bytes(build(b"x", 3)).unwrap();
        assert_eq!(region.chunk_nbt(1, 0).unwrap(), None);
        assert_eq!(region.chunk_nbt(32, 0).unwrap(), None);
    }

    #[test]
    fn rejects_small_file() {
        assert!(matches!(Region::from_bytes(vec![0; 100]), Err(RegionError::TooSmall)));
    }
}
