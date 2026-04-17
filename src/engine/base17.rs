/// Septidecimal (Base-17) Codec — AmniShunt wire format
/// Encoding: each byte (0-255) → 2 sept digits. Values 256-288 are metadata slots.

pub const RADIX: u16 = 17;
pub const PAIR_MAX: u16 = 289;
pub const META_START: u16 = 256;

// Metadata slot constants
pub const STREAM_START: u16 = 256;
pub const STREAM_END: u16 = 257;
pub const CHECKSUM: u16 = 258;
pub const HEARTBEAT: u16 = 259;
pub const ERROR_SLOT: u16 = 260;

const DIGITS: [u8; 17] = *b"0123456789ABCDEFG";
const INVALID: u8 = 0xFF;

static DECODE_TABLE: [u8; 128] = {
    let mut t = [INVALID; 128];
    t[b'0' as usize] = 0;  t[b'1' as usize] = 1;  t[b'2' as usize] = 2;
    t[b'3' as usize] = 3;  t[b'4' as usize] = 4;  t[b'5' as usize] = 5;
    t[b'6' as usize] = 6;  t[b'7' as usize] = 7;  t[b'8' as usize] = 8;
    t[b'9' as usize] = 9;  t[b'A' as usize] = 10; t[b'B' as usize] = 11;
    t[b'C' as usize] = 12; t[b'D' as usize] = 13; t[b'E' as usize] = 14;
    t[b'F' as usize] = 15; t[b'G' as usize] = 16;
    t[b'a' as usize] = 10; t[b'b' as usize] = 11; t[b'c' as usize] = 12;
    t[b'd' as usize] = 13; t[b'e' as usize] = 14; t[b'f' as usize] = 15;
    t[b'g' as usize] = 16;
    t
};

#[derive(Debug, Clone, PartialEq)]
pub enum CodecError {
    InvalidDigit(u8),
    OddLength,
    IncompleteVarint,
    ChecksumMismatch { expected: u8, got: u8 },
    BufferTooShort,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidDigit(d) => write!(f, "invalid sept digit: 0x{:02X}", d),
            Self::OddLength => write!(f, "odd-length sept string"),
            Self::IncompleteVarint => write!(f, "incomplete varint"),
            Self::ChecksumMismatch { expected, got } => write!(f, "checksum: expected {}, got {}", expected, got),
            Self::BufferTooShort => write!(f, "buffer too short"),
        }
    }
}

#[inline]
fn digit_val(ch: u8) -> Result<u16, CodecError> {
    if ch < 128 {
        let v = DECODE_TABLE[ch as usize];
        if v != INVALID { return Ok(v as u16); }
    }
    Err(CodecError::InvalidDigit(ch))
}

// --- Core encode/decode ---

pub fn encode_byte(b: u8) -> (u8, u8) {
    let v = b as u16;
    (DIGITS[(v / RADIX) as usize], DIGITS[(v % RADIX) as usize])
}

pub fn decode_pair(hi: u8, lo: u8) -> Result<u16, CodecError> {
    Ok(digit_val(hi)? * RADIX + digit_val(lo)?)
}

pub fn encode_meta(slot: u16) -> (u8, u8) {
    debug_assert!(slot >= META_START && slot < PAIR_MAX);
    let v = slot;
    (DIGITS[(v / RADIX) as usize], DIGITS[(v % RADIX) as usize])
}

// --- Byte buffer encode/decode ---

pub fn encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2);
    for &b in data {
        let (hi, lo) = encode_byte(b);
        out.push(hi);
        out.push(lo);
    }
    out
}

pub fn decode_bytes(sept: &[u8]) -> Result<Vec<u8>, CodecError> {
    if sept.len() % 2 != 0 { return Err(CodecError::OddLength); }
    let mut out = Vec::with_capacity(sept.len() / 2);
    for pair in sept.chunks_exact(2) {
        let val = decode_pair(pair[0], pair[1])?;
        if val >= META_START {
            continue; // skip metadata in raw decode
        }
        out.push(val as u8);
    }
    Ok(out)
}

// --- Varint (LEB128-style in base-17) ---
// Each sept digit: low 4 values (0-15) are payload nybble, high bit (>=8) means continue.
// Actually simpler: encode u64 as big-endian bytes, then base17 the bytes.
// Length prefix: first 2 sept digits = byte count (0-255), then that many encoded bytes.

pub fn encode_varint(val: u64) -> Vec<u8> {
    if val == 0 { return vec![DIGITS[0], DIGITS[0]]; } // length=0, meaning value 0
    let bytes = val.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    let significant = &bytes[start..];
    let mut out = Vec::with_capacity(2 + significant.len() * 2);
    let (lh, ll) = encode_byte(significant.len() as u8);
    out.push(lh); out.push(ll);
    for &b in significant {
        let (h, l) = encode_byte(b);
        out.push(h); out.push(l);
    }
    out
}

pub fn decode_varint(sept: &[u8]) -> Result<(u64, usize), CodecError> {
    if sept.len() < 2 { return Err(CodecError::BufferTooShort); }
    let len = decode_pair(sept[0], sept[1])? as usize;
    if len == 0 { return Ok((0, 2)); }
    let needed = 2 + len * 2;
    if sept.len() < needed { return Err(CodecError::BufferTooShort); }
    let mut val = 0u64;
    for i in 0..len {
        let b = decode_pair(sept[2 + i * 2], sept[2 + i * 2 + 1])?;
        if b >= META_START { return Err(CodecError::InvalidDigit(sept[2 + i * 2])); }
        val = (val << 8) | b as u64;
    }
    Ok((val, needed))
}

// --- Fixed-width encoders ---

pub fn encode_u16(v: u16) -> [u8; 4] {
    let hi = (v >> 8) as u8;
    let lo = (v & 0xFF) as u8;
    let (a, b) = encode_byte(hi);
    let (c, d) = encode_byte(lo);
    [a, b, c, d]
}

pub fn decode_u16(sept: &[u8]) -> Result<(u16, usize), CodecError> {
    if sept.len() < 4 { return Err(CodecError::BufferTooShort); }
    let hi = decode_pair(sept[0], sept[1])? as u16;
    let lo = decode_pair(sept[2], sept[3])? as u16;
    Ok(((hi << 8) | lo, 4))
}

pub fn encode_u32(v: u32) -> [u8; 8] {
    let b = v.to_be_bytes();
    let (a0, a1) = encode_byte(b[0]);
    let (b0, b1) = encode_byte(b[1]);
    let (c0, c1) = encode_byte(b[2]);
    let (d0, d1) = encode_byte(b[3]);
    [a0, a1, b0, b1, c0, c1, d0, d1]
}

pub fn decode_u32(sept: &[u8]) -> Result<(u32, usize), CodecError> {
    if sept.len() < 8 { return Err(CodecError::BufferTooShort); }
    let mut val = 0u32;
    for i in 0..4 {
        let b = decode_pair(sept[i * 2], sept[i * 2 + 1])?;
        if b >= META_START { return Err(CodecError::InvalidDigit(sept[i * 2])); }
        val = (val << 8) | b as u32;
    }
    Ok((val, 8))
}

pub fn encode_f32(v: f32) -> [u8; 8] { encode_u32(v.to_bits()) }

pub fn decode_f32(sept: &[u8]) -> Result<(f32, usize), CodecError> {
    let (bits, consumed) = decode_u32(sept)?;
    Ok((f32::from_bits(bits), consumed))
}

// --- String encoding: u16 length + encoded bytes ---

pub fn encode_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let len = bytes.len().min(u16::MAX as usize) as u16;
    let mut out = Vec::with_capacity(4 + len as usize * 2);
    out.extend_from_slice(&encode_u16(len));
    out.extend_from_slice(&encode_bytes(&bytes[..len as usize]));
    out
}

pub fn decode_string(sept: &[u8]) -> Result<(String, usize), CodecError> {
    let (len, _) = decode_u16(sept)?;
    let needed = 4 + len as usize * 2;
    if sept.len() < needed { return Err(CodecError::BufferTooShort); }
    let decoded = decode_bytes(&sept[4..needed])?;
    let s = String::from_utf8(decoded).map_err(|_| CodecError::InvalidDigit(0))?;
    Ok((s, needed))
}

// --- Checksum ---

pub struct Checksummer { sum: u16 }

impl Checksummer {
    pub fn new() -> Self { Self { sum: 0 } }

    pub fn feed(&mut self, digit: u8) {
        if let Ok(v) = digit_val(digit) { self.sum = (self.sum + v) % RADIX; }
    }

    pub fn feed_slice(&mut self, digits: &[u8]) {
        for &d in digits { self.feed(d); }
    }

    pub fn value(&self) -> u8 { self.sum as u8 }

    pub fn encode(&self) -> (u8, u8, u8, u8) {
        let (mh, ml) = encode_meta(CHECKSUM);
        let check_digit = DIGITS[self.sum as usize];
        (mh, ml, check_digit, DIGITS[0]) // pad to pair
    }

    pub fn reset(&mut self) { self.sum = 0; }
}

// --- Stream encoder/decoder ---

pub struct StreamEncoder {
    buffer: Vec<u8>,
    checksum: Checksummer,
}

impl StreamEncoder {
    pub fn new() -> Self {
        let mut enc = Self { buffer: Vec::with_capacity(4096), checksum: Checksummer::new() };
        let (h, l) = encode_meta(STREAM_START);
        enc.buffer.push(h); enc.buffer.push(l);
        enc
    }

    pub fn write_byte(&mut self, b: u8) {
        let (h, l) = encode_byte(b);
        self.checksum.feed(h); self.checksum.feed(l);
        self.buffer.push(h); self.buffer.push(l);
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        for &b in data { self.write_byte(b); }
    }

    pub fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_be_bytes());
    }

    pub fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_be_bytes());
    }

    pub fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    pub fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len().min(u16::MAX as usize) as u16;
        self.write_u16(len);
        self.write_bytes(&bytes[..len as usize]);
    }

    pub fn write_heartbeat(&mut self) {
        let (h, l) = encode_meta(HEARTBEAT);
        self.buffer.push(h); self.buffer.push(l);
    }

    pub fn finish(mut self) -> Vec<u8> {
        let (mh, ml, cv, cp) = self.checksum.encode();
        self.buffer.extend_from_slice(&[mh, ml, cv, cp]);
        let (eh, el) = encode_meta(STREAM_END);
        self.buffer.push(eh); self.buffer.push(el);
        self.buffer
    }

    pub fn len(&self) -> usize { self.buffer.len() }
}

pub struct StreamDecoder<'a> {
    data: &'a [u8],
    pos: usize,
    checksum: Checksummer,
}

impl<'a> StreamDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, CodecError> {
        if data.len() < 2 { return Err(CodecError::BufferTooShort); }
        let v = decode_pair(data[0], data[1])?;
        if v != STREAM_START { return Err(CodecError::InvalidDigit(data[0])); }
        Ok(Self { data, pos: 2, checksum: Checksummer::new() })
    }

    pub fn read_byte(&mut self) -> Result<u8, CodecError> {
        if self.pos + 2 > self.data.len() { return Err(CodecError::BufferTooShort); }
        let val = decode_pair(self.data[self.pos], self.data[self.pos + 1])?;
        if val >= META_START {
            self.pos += 2;
            return Err(CodecError::InvalidDigit(self.data[self.pos - 2]));
        }
        self.checksum.feed(self.data[self.pos]);
        self.checksum.feed(self.data[self.pos + 1]);
        self.pos += 2;
        Ok(val as u8)
    }

    pub fn read_u16(&mut self) -> Result<u16, CodecError> {
        let hi = self.read_byte()? as u16;
        let lo = self.read_byte()? as u16;
        Ok((hi << 8) | lo)
    }

    pub fn read_u32(&mut self) -> Result<u32, CodecError> {
        let mut val = 0u32;
        for _ in 0..4 { val = (val << 8) | self.read_byte()? as u32; }
        Ok(val)
    }

    pub fn read_f32(&mut self) -> Result<f32, CodecError> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_string(&mut self) -> Result<String, CodecError> {
        let len = self.read_u16()? as usize;
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len { bytes.push(self.read_byte()?); }
        String::from_utf8(bytes).map_err(|_| CodecError::InvalidDigit(0))
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, CodecError> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n { out.push(self.read_byte()?); }
        Ok(out)
    }

    pub fn remaining(&self) -> usize {
        if self.pos >= self.data.len() { 0 } else { (self.data.len() - self.pos) / 2 }
    }

    pub fn is_at_end(&self) -> bool {
        if self.pos + 2 > self.data.len() { return true; }
        if let Ok(v) = decode_pair(self.data[self.pos], self.data[self.pos + 1]) {
            v == STREAM_END || v == CHECKSUM
        } else { true }
    }

    pub fn peek_meta(&self) -> Option<u16> {
        if self.pos + 2 > self.data.len() { return None; }
        decode_pair(self.data[self.pos], self.data[self.pos + 1]).ok().filter(|&v| v >= META_START)
    }

    pub fn skip_meta(&mut self) { if self.peek_meta().is_some() { self.pos += 2; } }

    pub fn verify_checksum(&mut self) -> Result<bool, CodecError> {
        if self.pos + 4 > self.data.len() { return Ok(true); } // no checksum present
        let v = decode_pair(self.data[self.pos], self.data[self.pos + 1])?;
        if v != CHECKSUM { return Ok(true); }
        self.pos += 2;
        let expected = digit_val(self.data[self.pos])? as u8;
        self.pos += 2; // skip pad
        let got = self.checksum.value();
        if expected != got { return Err(CodecError::ChecksumMismatch { expected, got }); }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_bytes() {
        for b in 0..=255u8 {
            let (hi, lo) = encode_byte(b);
            let decoded = decode_pair(hi, lo).unwrap();
            assert_eq!(decoded, b as u16);
        }
    }

    #[test]
    fn roundtrip_buffer() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_bytes(&data);
        let decoded = decode_bytes(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn roundtrip_u32() {
        let vals = [0u32, 1, 255, 256, 65535, 0xDEADBEEF, u32::MAX];
        for &v in &vals {
            let enc = encode_u32(v);
            let (dec, consumed) = decode_u32(&enc).unwrap();
            assert_eq!(dec, v);
            assert_eq!(consumed, 8);
        }
    }

    #[test]
    fn roundtrip_string() {
        let s = "Hello, AmniShunt!";
        let enc = encode_string(s);
        let (dec, _) = decode_string(&enc).unwrap();
        assert_eq!(dec, s);
    }

    #[test]
    fn stream_roundtrip() {
        let mut enc = StreamEncoder::new();
        enc.write_u32(42);
        enc.write_string("test");
        enc.write_f32(3.14);
        let wire = enc.finish();

        let mut dec = StreamDecoder::new(&wire).unwrap();
        assert_eq!(dec.read_u32().unwrap(), 42);
        assert_eq!(dec.read_string().unwrap(), "test");
        let f = dec.read_f32().unwrap();
        assert!((f - 3.14).abs() < 0.001);
    }

    #[test]
    fn varint_roundtrip() {
        let vals = [0u64, 1, 127, 128, 255, 256, 65535, 0xFFFFFFFF, u64::MAX];
        for &v in &vals {
            let enc = encode_varint(v);
            let (dec, _) = decode_varint(&enc).unwrap();
            assert_eq!(dec, v, "failed for {}", v);
        }
    }

    #[test]
    fn expansion_factor() {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let encoded = encode_bytes(&data);
        let ratio = encoded.len() as f64 / data.len() as f64;
        assert!(ratio < 2.1, "expansion {} too high", ratio); // exactly 2.0x
    }
}
