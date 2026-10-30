//! Parser NBT (format Minecraft Java, big-endian).
//! Dipakai untuk membaca data chunk dari region file.

use std::fmt;

const MAX_DEPTH: u32 = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Value>),
    Compound(Vec<(String, Value)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Debug, PartialEq)]
pub enum NbtError {
    Eof,
    BadTag(u8),
    BadUtf8,
    BadLength,
    TooDeep,
}

impl fmt::Display for NbtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtError::Eof => write!(f, "data NBT habis sebelum waktunya"),
            NbtError::BadTag(t) => write!(f, "tag NBT tidak dikenal: {t}"),
            NbtError::BadUtf8 => write!(f, "string NBT bukan UTF-8 valid"),
            NbtError::BadLength => write!(f, "panjang NBT negatif"),
            NbtError::TooDeep => write!(f, "NBT terlalu dalam"),
        }
    }
}

impl Value {
    /// Mengambil field dari compound berdasarkan nama.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Compound(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Mengurai satu tag NBT root. Mengembalikan (nama root, nilai).
/// Root harus compound (tag 10), sesuai aturan format chunk.
pub fn parse(bytes: &[u8]) -> Result<(String, Value), NbtError> {
    let mut r = Reader { buf: bytes, pos: 0 };
    let tag = r.u8()?;
    if tag != 10 {
        return Err(NbtError::BadTag(tag));
    }
    let name = r.string()?;
    let value = read_payload(&mut r, 10, 0)?;
    Ok((name, value))
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], NbtError> {
        let end = self.pos.checked_add(n).ok_or(NbtError::Eof)?;
        if end > self.buf.len() {
            return Err(NbtError::Eof);
        }
        let slice = &self.buf[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, NbtError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, NbtError> {
        let b: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(u16::from_be_bytes(b))
    }

    fn i16(&mut self) -> Result<i16, NbtError> {
        let b: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(i16::from_be_bytes(b))
    }

    fn i32(&mut self) -> Result<i32, NbtError> {
        let b: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(i32::from_be_bytes(b))
    }

    fn i64(&mut self) -> Result<i64, NbtError> {
        let b: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(i64::from_be_bytes(b))
    }

    /// Panjang array atau list (i32, tidak boleh negatif).
    fn len(&mut self) -> Result<usize, NbtError> {
        let n = self.i32()?;
        if n < 0 {
            return Err(NbtError::BadLength);
        }
        Ok(n as usize)
    }

    fn string(&mut self) -> Result<String, NbtError> {
        let n = self.u16()? as usize;
        let bytes = self.take(n)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| NbtError::BadUtf8)
    }
}

fn read_payload(r: &mut Reader, tag: u8, depth: u32) -> Result<Value, NbtError> {
    if depth > MAX_DEPTH {
        return Err(NbtError::TooDeep);
    }
    Ok(match tag {
        1 => Value::Byte(r.u8()? as i8),
        2 => Value::Short(r.i16()?),
        3 => Value::Int(r.i32()?),
        4 => Value::Long(r.i64()?),
        5 => Value::Float(f32::from_bits(r.i32()? as u32)),
        6 => Value::Double(f64::from_bits(r.i64()? as u64)),
        7 => {
            let n = r.len()?;
            let bytes = r.take(n)?;
            Value::ByteArray(bytes.iter().map(|&b| b as i8).collect())
        }
        8 => Value::String(r.string()?),
        9 => {
            let elem = r.u8()?;
            let n = r.len()?;
            if elem == 0 && n > 0 {
                return Err(NbtError::BadTag(0));
            }
            let mut items = Vec::with_capacity(n.min(r.remaining()));
            for _ in 0..n {
                items.push(read_payload(r, elem, depth + 1)?);
            }
            Value::List(items)
        }
        10 => {
            let mut entries = Vec::new();
            loop {
                let t = r.u8()?;
                if t == 0 {
                    break;
                }
                let name = r.string()?;
                entries.push((name, read_payload(r, t, depth + 1)?));
            }
            Value::Compound(entries)
        }
        11 => {
            let n = r.len()?;
            let mut v = Vec::with_capacity(n.min(r.remaining() / 4));
            for _ in 0..n {
                v.push(r.i32()?);
            }
            Value::IntArray(v)
        }
        12 => {
            let n = r.len()?;
            let mut v = Vec::with_capacity(n.min(r.remaining() / 8));
            for _ in 0..n {
                v.push(r.i64()?);
            }
            Value::LongArray(v)
        }
        other => return Err(NbtError::BadTag(other)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_small_compound() {
        // Compound root "" berisi: Int "a" = 5, String "b" = "hi"
        let bytes = [
            0x0A, 0x00, 0x00, // root compound, nama kosong
            0x03, 0x00, 0x01, b'a', 0x00, 0x00, 0x00, 0x05, // Int a = 5
            0x08, 0x00, 0x01, b'b', 0x00, 0x02, b'h', b'i', // String b = "hi"
            0x00, // end
        ];
        let (name, v) = parse(&bytes).unwrap();
        assert_eq!(name, "");
        assert_eq!(v.get("a").and_then(Value::as_int), Some(5));
        assert_eq!(v.get("b").and_then(Value::as_str), Some("hi"));
        assert!(v.get("c").is_none());
    }

    #[test]
    fn parses_list_of_ints() {
        // Compound root berisi List "xs" tipe Int berisi [1, 2]
        let bytes = [
            0x0A, 0x00, 0x00,
            0x09, 0x00, 0x02, b'x', b's', 0x03, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x02,
            0x00,
        ];
        let (_, v) = parse(&bytes).unwrap();
        let list = v.get("xs").and_then(Value::as_list).unwrap();
        assert_eq!(list, &[Value::Int(1), Value::Int(2)]);
    }

    #[test]
    fn rejects_non_compound_root() {
        assert_eq!(parse(&[0x03, 0x00, 0x00]), Err(NbtError::BadTag(3)));
    }

    #[test]
    fn rejects_truncated_input() {
        assert_eq!(parse(&[0x0A, 0x00]), Err(NbtError::Eof));
    }
}
