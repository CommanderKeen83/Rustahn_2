use crate::ecs::World;
use crate::types::{EntityID, EntityType, SpriteID};
use crate::components::{Collider, PhysicsData};
use sola_raylib::prelude::{Rectangle, Vector2};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Lightweight Zero-Dependency JSON Parser
// ============================================================================

/// Minimal JSON value representation for zero-dependency map loading.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|n| n as f32)
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_f64().and_then(|n| {
            if n >= 0.0 && n <= u32::MAX as f64 {
                Some(n as u32)
            } else {
                None
            }
        })
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.as_object().and_then(|map| map.get(key))
    }
}

/// Recursive descent JSON parser.
struct JsonParser<'a> {
    chars: &'a [u8],
    cursor: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.as_bytes(),
            cursor: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.cursor < self.chars.len() {
            match self.chars[self.cursor] {
                b' ' | b'\t' | b'\n' | b'\r' => self.cursor += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.chars.get(self.cursor).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        let b = self.chars.get(self.cursor).copied();
        if b.is_some() {
            self.cursor += 1;
        }
        b
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(ch) => Err(format!(
                "Unexpected character '{}' at byte position {}",
                ch as char, self.cursor
            )),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.cursor += 1; // consume '{'
        let mut map = HashMap::new();

        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.cursor += 1;
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = match self.peek() {
                Some(b'"') => self.parse_string()?,
                _ => return Err(format!("Expected string key in object at position {}", self.cursor)),
            };

            self.skip_whitespace();
            if self.next_byte() != Some(b':') {
                return Err(format!("Expected ':' after key \"{}\" at position {}", key, self.cursor));
            }

            let val = self.parse_value()?;
            map.insert(key, val);

            self.skip_whitespace();
            match self.next_byte() {
                Some(b',') => continue,
                Some(b'}') => break,
                Some(ch) => {
                    return Err(format!(
                        "Expected ',' or '}}' in object, found '{}' at position {}",
                        ch as char, self.cursor
                    ))
                }
                None => return Err("Unexpected end of input in object".to_string()),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.cursor += 1; // consume '['
        let mut arr = Vec::new();

        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.cursor += 1;
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let val = self.parse_value()?;
            arr.push(val);

            self.skip_whitespace();
            match self.next_byte() {
                Some(b',') => continue,
                Some(b']') => break,
                Some(ch) => {
                    return Err(format!(
                        "Expected ',' or ']' in array, found '{}' at position {}",
                        ch as char, self.cursor
                    ))
                }
                None => return Err("Unexpected end of input in array".to_string()),
            }
        }

        Ok(JsonValue::Array(arr))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.cursor += 1; // consume opening '"'
        let mut buf: Vec<u8> = Vec::new();

        while let Some(b) = self.next_byte() {
            match b {
                b'"' => {
                    return String::from_utf8(buf)
                        .map_err(|e| format!("Invalid UTF-8 in string: {}", e));
                }
                b'\\' => {
                    let esc = self.next_byte().ok_or("Unexpected end of string escape")?;
                    match esc {
                        b'"' => buf.push(b'"'),
                        b'\\' => buf.push(b'\\'),
                        b'/' => buf.push(b'/'),
                        b'b' => buf.push(0x08),
                        b'f' => buf.push(0x0C),
                        b'n' => buf.push(b'\n'),
                        b'r' => buf.push(b'\r'),
                        b't' => buf.push(b'\t'),
                        b'u' => {
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                let h = self.next_byte().ok_or("Incomplete unicode escape")?;
                                hex.push(h as char);
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|e| format!("Invalid unicode escape \\u{}: {}", hex, e))?;
                            let ch = char::from_u32(code).ok_or("Invalid unicode scalar value")?;
                            let mut tmp = [0u8; 4];
                            buf.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
                        }
                        other => {
                            return Err(format!(
                                "Unknown escape sequence '\\{}' at position {}",
                                other as char, self.cursor
                            ))
                        }
                    }
                }
                normal => buf.push(normal),
            }
        }

        Err("Unterminated string literal".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.cursor;
        if self.peek() == Some(b'-') {
            self.cursor += 1;
        }

        while let Some(b) = self.peek() {
            match b {
                b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-' => self.cursor += 1,
                _ => break,
            }
        }

        let num_str = std::str::from_utf8(&self.chars[start..self.cursor])
            .map_err(|e| format!("Invalid UTF-8 in number: {}", e))?;

        num_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|e| format!("Invalid number \"{}\" at position {}: {}", num_str, start, e))
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.chars[self.cursor..].starts_with(b"true") {
            self.cursor += 4;
            Ok(JsonValue::Bool(true))
        } else if self.chars[self.cursor..].starts_with(b"false") {
            self.cursor += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(format!("Invalid boolean token at position {}", self.cursor))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.chars[self.cursor..].starts_with(b"null") {
            self.cursor += 4;
            Ok(JsonValue::Null)
        } else {
            Err(format!("Invalid null token at position {}", self.cursor))
        }
    }
}

// ============================================================================
// Map Types & Error Definitions
// ============================================================================

/// Errors that can occur when loading or parsing a level map.
#[derive(Debug, Clone, PartialEq)]
pub enum MapError {
    IoError(String),
    ParseError(String),
    MissingField(String),
    InvalidFormat(String),
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapError::IoError(msg) => write!(f, "Map I/O error: {}", msg),
            MapError::ParseError(msg) => write!(f, "Map JSON parse error: {}", msg),
            MapError::MissingField(field) => write!(f, "Map missing required field: '{}'", field),
            MapError::InvalidFormat(msg) => write!(f, "Invalid map format: {}", msg),
        }
    }
}

impl std::error::Error for MapError {}

/// Known types of objects in a Tiled map.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapObjectType {
    Platform,
    Spawn,
    Exit,
    Gobbler,
    Other(String),
}

impl MapObjectType {
    pub fn from_str_type(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "platform" => MapObjectType::Platform,
            "spawn" => MapObjectType::Spawn,
            "exit" => MapObjectType::Exit,
            "gobbler" => MapObjectType::Gobbler,
            _ => MapObjectType::Other(s.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            MapObjectType::Platform => "Platform",
            MapObjectType::Spawn => "Spawn",
            MapObjectType::Exit => "Exit",
            MapObjectType::Gobbler => "Gobbler",
            MapObjectType::Other(s) => s.as_str(),
        }
    }
}

/// A single geometric object defined inside a Tiled object layer.
#[derive(Debug, Clone, PartialEq)]
pub struct MapObject {
    pub id: u32,
    pub name: String,
    pub object_type: MapObjectType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub visible: bool,
}

impl MapObject {
    /// World center position of this object.
    ///
    /// For rectangle objects (width > 0 or height > 0), Tiled coordinates (x, y) specify
    /// the top-left corner, so center is `(x + width / 2, y + height / 2)`.
    /// For point objects (such as `Spawn` markers with 0 width and height), returns `(x, y)`.
    pub fn center(&self) -> Vector2 {
        if self.width <= 0.0 && self.height <= 0.0 {
            Vector2::new(self.x, self.y)
        } else if self.rotation != 0.0 {
            let (s, c) = self.rotation.to_radians().sin_cos();
            let (hx, hy) = (self.width * 0.5, self.height * 0.5);
            Vector2::new(self.x + hx * c - hy * s, self.y + hx * s + hy * c)
        } else {
            Vector2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
        }
    }

    /// World rectangle bounds `(x, y, width, height)`.
    pub fn bounds(&self) -> Rectangle {
        Rectangle::new(self.x, self.y, self.width, self.height)
    }

    pub fn is_platform(&self) -> bool {
        self.object_type == MapObjectType::Platform
    }

    pub fn is_spawn(&self) -> bool {
        self.object_type == MapObjectType::Spawn
    }

    pub fn is_exit(&self) -> bool {
        self.object_type == MapObjectType::Exit
    }

    pub fn is_gobbler(&self) -> bool {
        self.object_type == MapObjectType::Gobbler
    }
}

// ============================================================================
// Base64 & DEFLATE / Zlib Decompression for Tiled Tile Layers
// ============================================================================

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;

    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\t' | b'\r' | b'\n' => continue,
            other => {
                return Err(format!(
                    "Invalid base64 character '{}'",
                    other as char
                ))
            }
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }

    Ok(out)
}

struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    fn read_bits(&mut self, count: u8) -> Result<u32, String> {
        let mut result: u32 = 0;
        for i in 0..count {
            if self.byte_pos >= self.data.len() {
                return Err("Unexpected end of compressed DEFLATE stream".to_string());
            }
            let bit = (self.data[self.byte_pos] >> self.bit_pos) & 1;
            result |= (bit as u32) << i;
            self.bit_pos += 1;
            if self.bit_pos == 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }
        Ok(result)
    }

    fn align_to_byte(&mut self) {
        if self.bit_pos > 0 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
    }

    fn read_u16_le(&mut self) -> Result<u16, String> {
        self.align_to_byte();
        if self.byte_pos + 2 > self.data.len() {
            return Err("Unexpected end of stream reading u16".to_string());
        }
        let val = u16::from_le_bytes([self.data[self.byte_pos], self.data[self.byte_pos + 1]]);
        self.byte_pos += 2;
        Ok(val)
    }
}

struct HuffmanTable {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl HuffmanTable {
    fn from_code_lengths(lengths: &[u8]) -> Result<Self, String> {
        let mut counts = [0u16; 16];
        for &len in lengths {
            if len as usize >= 16 {
                return Err(format!("Huffman code length {} exceeds maximum 15", len));
            }
            if len > 0 {
                counts[len as usize] += 1;
            }
        }

        let mut offsets = [0u16; 16];
        let mut total = 0u16;
        for bits in 1..16 {
            offsets[bits] = total;
            total += counts[bits];
        }

        let mut symbols = vec![0u16; total as usize];
        for (sym, &len) in lengths.iter().enumerate() {
            if len > 0 {
                let idx = offsets[len as usize] as usize;
                offsets[len as usize] += 1;
                symbols[idx] = sym as u16;
            }
        }

        Ok(Self { counts, symbols })
    }

    fn decode(&self, reader: &mut BitReader<'_>) -> Result<u16, String> {
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;

        for len in 1..16 {
            let bit = reader.read_bits(1)? as i32;
            code |= bit;
            let count = self.counts[len] as i32;
            if code - count < first {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }

        Err("Invalid Huffman code in DEFLATE stream".to_string())
    }
}

fn inflate_raw(data: &[u8]) -> Result<Vec<u8>, String> {
    const LENGTH_BASE: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LENGTH_EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    const CL_ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];

    let mut reader = BitReader::new(data);
    let mut out = Vec::new();

    loop {
        let bfinal = reader.read_bits(1)?;
        let btype = reader.read_bits(2)?;

        match btype {
            0 => {
                let len = reader.read_u16_le()?;
                let nlen = reader.read_u16_le()?;
                if len != !nlen {
                    return Err("Stored DEFLATE block length check failed".to_string());
                }
                let count = len as usize;
                if reader.byte_pos + count > reader.data.len() {
                    return Err("Unexpected end of stream in stored DEFLATE block".to_string());
                }
                out.extend_from_slice(&reader.data[reader.byte_pos..reader.byte_pos + count]);
                reader.byte_pos += count;
            }
            1 | 2 => {
                let (lit_len_table, dist_table) = if btype == 1 {
                    let mut lit_lengths = [0u8; 288];
                    lit_lengths[0..=143].fill(8);
                    lit_lengths[144..=255].fill(9);
                    lit_lengths[256..=279].fill(7);
                    lit_lengths[280..=287].fill(8);
                    let dist_lengths = [5u8; 32];
                    (
                        HuffmanTable::from_code_lengths(&lit_lengths)?,
                        HuffmanTable::from_code_lengths(&dist_lengths)?,
                    )
                } else {
                    let hlit = reader.read_bits(5)? as usize + 257;
                    let hdist = reader.read_bits(5)? as usize + 1;
                    let hclen = reader.read_bits(4)? as usize + 4;

                    let mut cl_lengths = [0u8; 19];
                    for i in 0..hclen {
                        cl_lengths[CL_ORDER[i]] = reader.read_bits(3)? as u8;
                    }
                    let cl_table = HuffmanTable::from_code_lengths(&cl_lengths)?;

                    let mut lengths = Vec::with_capacity(hlit + hdist);
                    while lengths.len() < hlit + hdist {
                        let sym = cl_table.decode(&mut reader)?;
                        match sym {
                            0..=15 => lengths.push(sym as u8),
                            16 => {
                                let prev = *lengths
                                    .last()
                                    .ok_or("Repeat code 16 with no previous length")?;
                                let repeat = reader.read_bits(2)? as usize + 3;
                                for _ in 0..repeat {
                                    lengths.push(prev);
                                }
                            }
                            17 => {
                                let repeat = reader.read_bits(3)? as usize + 3;
                                for _ in 0..repeat {
                                    lengths.push(0);
                                }
                            }
                            18 => {
                                let repeat = reader.read_bits(7)? as usize + 11;
                                for _ in 0..repeat {
                                    lengths.push(0);
                                }
                            }
                            _ => return Err(format!("Invalid code length symbol {}", sym)),
                        }
                    }

                    if lengths.len() != hlit + hdist {
                        return Err("Code length run exceeded expected count".to_string());
                    }

                    (
                        HuffmanTable::from_code_lengths(&lengths[..hlit])?,
                        HuffmanTable::from_code_lengths(&lengths[hlit..])?,
                    )
                };

                loop {
                    let sym = lit_len_table.decode(&mut reader)?;
                    if sym < 256 {
                        out.push(sym as u8);
                    } else if sym == 256 {
                        break;
                    } else {
                        let len_idx = (sym - 257) as usize;
                        if len_idx >= LENGTH_BASE.len() {
                            return Err(format!("Invalid length symbol {}", sym));
                        }
                        let length = LENGTH_BASE[len_idx] as usize
                            + reader.read_bits(LENGTH_EXTRA[len_idx])? as usize;

                        let dist_sym = dist_table.decode(&mut reader)? as usize;
                        if dist_sym >= DIST_BASE.len() {
                            return Err(format!("Invalid distance symbol {}", dist_sym));
                        }
                        let distance = DIST_BASE[dist_sym] as usize
                            + reader.read_bits(DIST_EXTRA[dist_sym])? as usize;

                        if distance == 0 || distance > out.len() {
                            return Err(format!(
                                "Invalid back-reference distance {} (output len {})",
                                distance,
                                out.len()
                            ));
                        }

                        for _ in 0..length {
                            let b = out[out.len() - distance];
                            out.push(b);
                        }
                    }
                }
            }
            _ => return Err(format!("Invalid DEFLATE block type {}", btype)),
        }

        if bfinal != 0 {
            break;
        }
    }

    Ok(out)
}

fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 6 {
        return Err("Zlib stream too short".to_string());
    }
    let cmf = data[0];
    let flg = data[1];
    if (cmf & 0x0F) != 8 {
        return Err(format!("Unsupported zlib compression method {}", cmf & 0x0F));
    }
    if ((cmf as u16) * 256 + (flg as u16)) % 31 != 0 {
        return Err("Invalid zlib header checksum".to_string());
    }
    let start = if (flg & 0x20) != 0 { 6 } else { 2 };
    if start + 4 > data.len() {
        return Err("Zlib stream truncated".to_string());
    }
    inflate_raw(&data[start..data.len() - 4])
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 18 || data[0] != 0x1F || data[1] != 0x8B {
        return Err("Invalid gzip header".to_string());
    }
    if data[2] != 8 {
        return Err("Unsupported gzip compression method".to_string());
    }
    let flags = data[3];
    let mut pos = 10;
    if (flags & 0x04) != 0 {
        if pos + 2 > data.len() {
            return Err("Truncated gzip FEXTRA".to_string());
        }
        let xlen = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2 + xlen;
    }
    if (flags & 0x08) != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    if (flags & 0x10) != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    if (flags & 0x02) != 0 {
        pos += 2;
    }
    if pos + 8 > data.len() {
        return Err("Truncated gzip stream".to_string());
    }
    inflate_raw(&data[pos..data.len() - 8])
}

fn decode_tile_layer_data(layer_val: &JsonValue) -> Result<Vec<u32>, MapError> {
    let Some(data_val) = layer_val.get("data") else {
        return Ok(Vec::new());
    };

    // Case 1: Uncompressed JSON array of integer GIDs
    if let Some(arr) = data_val.as_array() {
        let mut gids = Vec::with_capacity(arr.len());
        for v in arr {
            let gid = v.as_u32().unwrap_or(0);
            gids.push(gid);
        }
        return Ok(gids);
    }

    // Case 2: Base64-encoded (optionally compressed) binary GID array
    if let Some(b64_str) = data_val.as_str() {
        let encoding = layer_val
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("base64");
        if encoding != "base64" {
            return Err(MapError::InvalidFormat(format!(
                "Unsupported tile layer encoding '{}'",
                encoding
            )));
        }

        let raw_bytes = decode_base64(b64_str).map_err(MapError::InvalidFormat)?;
        let compression = layer_val
            .get("compression")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let decompressed = match compression {
            "" => raw_bytes,
            "zlib" => decompress_zlib(&raw_bytes).map_err(MapError::InvalidFormat)?,
            "gzip" => decompress_gzip(&raw_bytes).map_err(MapError::InvalidFormat)?,
            other => {
                return Err(MapError::InvalidFormat(format!(
                    "Unsupported tile layer compression '{}'",
                    other
                )))
            }
        };

        if decompressed.len() % 4 != 0 {
            return Err(MapError::InvalidFormat(format!(
                "Decompressed tile data byte length ({}) is not a multiple of 4",
                decompressed.len()
            )));
        }

        let mut gids = Vec::with_capacity(decompressed.len() / 4);
        for chunk in decompressed.chunks_exact(4) {
            gids.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        return Ok(gids);
    }

    Ok(Vec::new())
}

fn normalize_tileset_image_path(raw_image: &str, map_dir: Option<&Path>) -> String {
    let cleaned = raw_image.replace('\\', "/");
    if let Some(dir) = map_dir {
        let joined = dir.join(&cleaned);
        let mut components = Vec::new();
        for comp in joined.components() {
            match comp {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    components.pop();
                }
                other => components.push(other),
            }
        }
        let normalized: PathBuf = components.into_iter().collect();
        return normalized.to_string_lossy().replace('\\', "/");
    }

    // Fallback when parsed directly from a JSON string without a known file path:
    // resolve "../../graphics/..." to "resources/graphics/..."
    let mut trimmed = cleaned.as_str();
    while let Some(rest) = trimmed.strip_prefix("../") {
        trimmed = rest;
    }
    if trimmed != cleaned {
        let candidate = format!("resources/{}", trimmed);
        if Path::new(&candidate).exists() || trimmed.starts_with("graphics/") {
            return candidate;
        }
    }

    cleaned
}

/// Tileset metadata referencing a sprite sheet image for tile layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTileset {
    pub first_gid: u32,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tile_count: u32,
    pub columns: u32,
    pub image: String,
    pub image_width: u32,
    pub image_height: u32,
}

impl MapTileset {
    pub const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x8000_0000;
    pub const FLIPPED_VERTICALLY_FLAG: u32 = 0x4000_0000;
    pub const FLIPPED_DIAGONALLY_FLAG: u32 = 0x2000_0000;
    pub const GID_MASK: u32 =
        !(Self::FLIPPED_HORIZONTALLY_FLAG | Self::FLIPPED_VERTICALLY_FLAG | Self::FLIPPED_DIAGONALLY_FLAG);

    /// Strips Tiled flip flags from a raw GID.
    pub fn strip_flags(gid: u32) -> u32 {
        gid & Self::GID_MASK
    }

    /// Returns true if the given GID belongs to this tileset.
    pub fn contains_gid(&self, gid: u32) -> bool {
        let raw_gid = Self::strip_flags(gid);
        if raw_gid < self.first_gid {
            return false;
        }
        let local_id = raw_gid - self.first_gid;
        self.tile_count == 0 || local_id < self.tile_count
    }

    /// Computes the source rectangle on the tileset sprite sheet for the given `gid`.
    /// Preserves horizontal and vertical flip flags as negative width/height for Raylib `draw_texture_pro`.
    pub fn tile_source_rect(&self, gid: u32) -> Option<Rectangle> {
        if !self.contains_gid(gid) {
            return None;
        }
        let raw_gid = Self::strip_flags(gid);
        let local_id = raw_gid - self.first_gid;

        let cols = if self.columns > 0 {
            self.columns
        } else if self.tile_width > 0 && self.image_width >= self.tile_width {
            (self.image_width.saturating_sub(2 * self.margin) + self.spacing)
                / (self.tile_width + self.spacing)
        } else {
            1
        };
        if cols == 0 {
            return None;
        }

        let col = local_id % cols;
        let row = local_id / cols;
        let x = (self.margin + col * (self.tile_width + self.spacing)) as f32;
        let y = (self.margin + row * (self.tile_height + self.spacing)) as f32;

        let mut w = self.tile_width as f32;
        let mut h = self.tile_height as f32;
        if (gid & Self::FLIPPED_HORIZONTALLY_FLAG) != 0 {
            w = -w;
        }
        if (gid & Self::FLIPPED_VERTICALLY_FLAG) != 0 {
            h = -h;
        }

        Some(Rectangle::new(x, y, w, h))
    }
}

/// Type of layer in a Tiled map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerType {
    ObjectGroup,
    TileLayer,
    Other(String),
}

impl LayerType {
    pub fn from_str_type(s: &str) -> Self {
        match s {
            "objectgroup" => LayerType::ObjectGroup,
            "tilelayer" => LayerType::TileLayer,
            other => LayerType::Other(other.to_string()),
        }
    }
}

/// A layer within a Tiled map containing objects or tile data.
#[derive(Debug, Clone, PartialEq)]
pub struct MapLayer {
    pub id: u32,
    pub name: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub opacity: f32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>,
    pub objects: Vec<MapObject>,
}

impl MapLayer {
    pub fn is_tile_layer(&self) -> bool {
        self.layer_type == LayerType::TileLayer
    }

    pub fn is_object_group(&self) -> bool {
        self.layer_type == LayerType::ObjectGroup
    }

    /// Returns the tile GID at `(col, row)`, or `0` if empty or out of bounds.
    pub fn tile_gid_at(&self, col: u32, row: u32) -> u32 {
        if col >= self.width || row >= self.height {
            return 0;
        }
        let idx = (row * self.width + col) as usize;
        self.data.get(idx).copied().unwrap_or(0)
    }

    /// Iterator over all non-empty tiles `(col, row, gid)` in this tile layer.
    pub fn non_empty_tiles(&self) -> impl Iterator<Item = (u32, u32, u32)> + '_ {
        let w = self.width;
        self.data.iter().enumerate().filter_map(move |(idx, &gid)| {
            if gid != 0 && w > 0 {
                let col = (idx as u32) % w;
                let row = (idx as u32) / w;
                Some((col, row, gid))
            } else {
                None
            }
        })
    }
}

/// Level map loaded from a Tiled JSON map export.
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<MapLayer>,
    pub tilesets: Vec<MapTileset>,
}

/// Collection of ECS entities and markers generated when spawning a `Map` into a `World`.
#[derive(Debug, Clone, Default)]
pub struct MapSpawnResult {
    pub platforms: Vec<EntityID>,
    pub spawn_point: Option<Vector2>,
    pub exits: Vec<EntityID>,
    pub gobblers: Vec<EntityID>,
}

impl Map {
    /// Total map pixel width (`width * tile_width`).
    pub fn pixel_width(&self) -> f32 {
        (self.width * self.tile_width) as f32
    }

    /// Total map pixel height (`height * tile_height`).
    pub fn pixel_height(&self) -> f32 {
        (self.height * self.tile_height) as f32
    }

    /// Map pixel bounds rectangle from (0, 0) to (pixel_width, pixel_height).
    pub fn bounds(&self) -> Rectangle {
        Rectangle::new(0.0, 0.0, self.pixel_width(), self.pixel_height())
    }

    /// Iterator over all tile layers in the map.
    pub fn tile_layers(&self) -> impl Iterator<Item = &MapLayer> {
        self.layers.iter().filter(|l| l.is_tile_layer())
    }

    /// Slice of all tilesets defined in the map.
    pub fn tilesets(&self) -> &[MapTileset] {
        &self.tilesets
    }

    /// Finds the tileset that owns the given `gid`.
    pub fn find_tileset_for_gid(&self, gid: u32) -> Option<&MapTileset> {
        let raw_gid = MapTileset::strip_flags(gid);
        if raw_gid == 0 {
            return None;
        }
        self.tilesets
            .iter()
            .filter(|ts| ts.contains_gid(gid))
            .max_by_key(|ts| ts.first_gid)
    }

    /// Computes the source rectangle in the corresponding tileset for `gid`.
    pub fn tile_source_rect(&self, gid: u32) -> Option<Rectangle> {
        self.find_tileset_for_gid(gid)
            .and_then(|ts| ts.tile_source_rect(gid))
    }

    /// Iterator over all objects across all layers.
    pub fn all_objects(&self) -> impl Iterator<Item = &MapObject> {
        self.layers.iter().flat_map(|l| l.objects.iter())
    }

    /// Iterator over all platform objects.
    pub fn platforms(&self) -> impl Iterator<Item = &MapObject> {
        self.all_objects().filter(|o| o.is_platform())
    }

    /// Iterator over all exit objects.
    pub fn exits(&self) -> impl Iterator<Item = &MapObject> {
        self.all_objects().filter(|o| o.is_exit())
    }

    /// Iterator over all Gobbler enemy spawn objects.
    pub fn gobblers(&self) -> impl Iterator<Item = &MapObject> {
        self.all_objects().filter(|o| o.is_gobbler())
    }

    /// Player spawn point, if defined in the map objects.
    pub fn spawn_point(&self) -> Option<Vector2> {
        self.all_objects().find(|o| o.is_spawn()).map(|o| o.center())
    }

    /// Loads a Tiled JSON map from a file path.
    ///
    /// Resilient: if `path` is not directly found, attempts appending `.json` or stripping
    /// `.json`, allowing both `map1` and `map1.json` to work transparently.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, MapError> {
        let p = path.as_ref();
        let resolved = Self::resolve_file_path(p)?;
        let content = fs::read_to_string(&resolved).map_err(|e| {
            MapError::IoError(format!("Failed to read '{}': {}", resolved.display(), e))
        })?;
        Self::from_json_str_with_dir(&content, resolved.parent())
    }

    /// Resolves file paths with or without `.json` extension.
    fn resolve_file_path(path: &Path) -> Result<PathBuf, MapError> {
        if path.exists() && path.is_file() {
            return Ok(path.to_path_buf());
        }

        // Try with .json appended
        let mut with_json = path.to_path_buf();
        with_json.set_extension("json");
        if with_json.exists() && with_json.is_file() {
            return Ok(with_json);
        }

        // Try stripping .json if present
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let without_json = path.with_extension("");
            if without_json.exists() && without_json.is_file() {
                return Ok(without_json);
            }
        }

        Err(MapError::IoError(format!(
            "Map file not found at '{}' (also checked '{}')",
            path.display(),
            with_json.display()
        )))
    }

    /// Parses a Tiled JSON map string.
    pub fn from_json_str(json_str: &str) -> Result<Self, MapError> {
        Self::from_json_str_with_dir(json_str, None)
    }

    fn from_json_str_with_dir(json_str: &str, map_dir: Option<&Path>) -> Result<Self, MapError> {
        let mut parser = JsonParser::new(json_str);
        let root = parser.parse_value().map_err(MapError::ParseError)?;

        let width = root
            .get("width")
            .and_then(|v| v.as_u32())
            .ok_or_else(|| MapError::MissingField("width".to_string()))?;

        let height = root
            .get("height")
            .and_then(|v| v.as_u32())
            .ok_or_else(|| MapError::MissingField("height".to_string()))?;

        let tile_width = root
            .get("tilewidth")
            .and_then(|v| v.as_u32())
            .ok_or_else(|| MapError::MissingField("tilewidth".to_string()))?;

        let tile_height = root
            .get("tileheight")
            .and_then(|v| v.as_u32())
            .ok_or_else(|| MapError::MissingField("tileheight".to_string()))?;

        let mut layers = Vec::new();
        if let Some(layer_array) = root.get("layers").and_then(|v| v.as_array()) {
            for l_val in layer_array {
                let id = l_val.get("id").and_then(|v| v.as_u32()).unwrap_or(0);
                let name = l_val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let type_str = l_val
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("objectgroup");
                let layer_type = LayerType::from_str_type(type_str);
                let visible = l_val.get("visible").and_then(|v| v.as_bool()).unwrap_or(true);
                let opacity = l_val.get("opacity").and_then(|v| v.as_f32()).unwrap_or(1.0);
                let layer_width = l_val.get("width").and_then(|v| v.as_u32()).unwrap_or(width);
                let layer_height = l_val.get("height").and_then(|v| v.as_u32()).unwrap_or(height);

                let data = if layer_type == LayerType::TileLayer {
                    decode_tile_layer_data(l_val)?
                } else {
                    Vec::new()
                };

                let mut objects = Vec::new();
                if let Some(obj_array) = l_val.get("objects").and_then(|v| v.as_array()) {
                    for o_val in obj_array {
                        let obj_id = o_val.get("id").and_then(|v| v.as_u32()).unwrap_or(0);
                        let obj_name = o_val
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Tiled stores type under "type" or "class"
                        let obj_type_str = o_val
                            .get("type")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                            .or_else(|| o_val.get("class").and_then(|v| v.as_str()).filter(|s| !s.is_empty()))
                            .unwrap_or("");
                        let object_type = MapObjectType::from_str_type(obj_type_str);

                        let x = o_val.get("x").and_then(|v| v.as_f32()).unwrap_or(0.0);
                        let y = o_val.get("y").and_then(|v| v.as_f32()).unwrap_or(0.0);
                        let w = o_val.get("width").and_then(|v| v.as_f32()).unwrap_or(0.0);
                        let h = o_val.get("height").and_then(|v| v.as_f32()).unwrap_or(0.0);
                        let rotation = o_val.get("rotation").and_then(|v| v.as_f32()).unwrap_or(0.0);
                        let obj_visible = o_val.get("visible").and_then(|v| v.as_bool()).unwrap_or(true);

                        objects.push(MapObject {
                            id: obj_id,
                            name: obj_name,
                            object_type,
                            x,
                            y,
                            width: w,
                            height: h,
                            rotation,
                            visible: obj_visible,
                        });
                    }
                }

                layers.push(MapLayer {
                    id,
                    name,
                    layer_type,
                    visible,
                    opacity,
                    width: layer_width,
                    height: layer_height,
                    data,
                    objects,
                });
            }
        }

        let mut tilesets = Vec::new();
        if let Some(ts_array) = root.get("tilesets").and_then(|v| v.as_array()) {
            for ts_val in ts_array {
                let first_gid = ts_val.get("firstgid").and_then(|v| v.as_u32()).unwrap_or(1);
                let name = ts_val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let ts_tile_width = ts_val
                    .get("tilewidth")
                    .and_then(|v| v.as_u32())
                    .unwrap_or(tile_width);
                let ts_tile_height = ts_val
                    .get("tileheight")
                    .and_then(|v| v.as_u32())
                    .unwrap_or(tile_height);
                let spacing = ts_val.get("spacing").and_then(|v| v.as_u32()).unwrap_or(0);
                let margin = ts_val.get("margin").and_then(|v| v.as_u32()).unwrap_or(0);
                let tile_count = ts_val.get("tilecount").and_then(|v| v.as_u32()).unwrap_or(0);
                let columns = ts_val.get("columns").and_then(|v| v.as_u32()).unwrap_or(0);
                let raw_image = ts_val.get("image").and_then(|v| v.as_str()).unwrap_or("");
                let image = normalize_tileset_image_path(raw_image, map_dir);
                let image_width = ts_val.get("imagewidth").and_then(|v| v.as_u32()).unwrap_or(0);
                let image_height = ts_val.get("imageheight").and_then(|v| v.as_u32()).unwrap_or(0);

                tilesets.push(MapTileset {
                    first_gid,
                    name,
                    tile_width: ts_tile_width,
                    tile_height: ts_tile_height,
                    spacing,
                    margin,
                    tile_count,
                    columns,
                    image,
                    image_width,
                    image_height,
                });
            }
        }

        Ok(Self {
            width,
            height,
            tile_width,
            tile_height,
            layers,
            tilesets,
        })
    }

    /// Spawns all platform and exit objects into the ECS `World`.
    ///
    /// - Platforms are spawned as solid `EntityType::Ground` entities with `PhysicsData::static_body()`
    ///   and a `Collider` matching the platform dimensions.
    /// - Exits are spawned as sensor `EntityType::Obstacle` entities.
    pub fn spawn_into_world(&self, world: &mut World) -> MapSpawnResult {
        let mut result = MapSpawnResult {
            platforms: Vec::new(),
            spawn_point: self.spawn_point(),
            exits: Vec::new(),
            gobblers: Vec::new(),
        };

        for obj in self.platforms() {
            let center = obj.center();
            let id = world
                .spawn(
                    EntityType::Ground,
                    center,
                    Vector2::new(0.0, 0.0),
                    obj.rotation,
                    true,
                    SpriteID::None,
                )
                .with_collider(Collider::new(obj.width, obj.height))
                .with_physics(PhysicsData::static_body())
                .build();
            result.platforms.push(id);
        }

        for obj in self.exits() {
            let center = obj.center();
            let id = world
                .spawn(
                    EntityType::Obstacle,
                    center,
                    Vector2::new(0.0, 0.0),
                    obj.rotation,
                    true,
                    SpriteID::None,
                )
                .with_collider(Collider::sensor(obj.width, obj.height))
                .with_physics(PhysicsData::static_body())
                .build();
            result.exits.push(id);
        }

        for obj in self.gobblers() {
            let center = obj.center();
            let width = if obj.width > 0.0 { obj.width } else { crate::components::GobblerData::WIDTH };
            let height = if obj.height > 0.0 { obj.height } else { crate::components::GobblerData::HEIGHT };
            let id = world
                .spawn(
                    EntityType::Gobbler,
                    center,
                    Vector2::new(0.0, 0.0),
                    obj.rotation,
                    true,
                    SpriteID::Gobbler,
                )
                .with_collider(Collider::new(width, height))
                .with_physics_preset(crate::components::PhysicsPreset::Normal)
                .with_gobbler(crate::components::GobblerData::DEFAULT_SPEED)
                .build();
            result.gobblers.push(id);
        }

        result
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TILED_JSON: &str = r#"{
 "compressionlevel":-1,
 "height":35,
 "infinite":false,
 "layers":[
        {
         "draworder":"topdown",
         "id":2,
         "name":"Platforms",
         "objects":[
                {
                 "height":64,
                 "id":2,
                 "name":"",
                 "opacity":1,
                 "rotation":0,
                 "type":"Platform",
                 "visible":true,
                 "width":416,
                 "x":0,
                 "y":672
                },
                {
                 "height":64,
                 "id":17,
                 "name":"",
                 "opacity":1,
                 "rotation":0,
                 "type":"Exit",
                 "visible":true,
                 "width":160,
                 "x":4320,
                 "y":192
                },
                {
                 "height":0,
                 "id":21,
                 "name":"",
                 "opacity":1,
                 "rotation":0,
                 "type":"Spawn",
                 "visible":true,
                 "width":0,
                 "x":160,
                 "y":544
                }],
         "opacity":1,
         "type":"objectgroup",
         "visible":true,
         "x":0,
         "y":0
        }],
 "nextlayerid":3,
 "nextobjectid":22,
 "orientation":"orthogonal",
 "renderorder":"right-down",
 "tiledversion":"1.12.2",
 "tileheight":32,
 "tilesets":[],
 "tilewidth":32,
 "type":"map",
 "version":"1.10",
 "width":140
}"#;

    #[test]
    fn test_parse_sample_tiled_json() {
        let map = Map::from_json_str(SAMPLE_TILED_JSON).expect("Failed to parse sample Tiled JSON");
        assert_eq!(map.width, 140);
        assert_eq!(map.height, 35);
        assert_eq!(map.tile_width, 32);
        assert_eq!(map.tile_height, 32);
        assert_eq!(map.pixel_width(), 4480.0);
        assert_eq!(map.pixel_height(), 1120.0);

        assert_eq!(map.layers.len(), 1);
        let layer = &map.layers[0];
        assert_eq!(layer.name, "Platforms");
        assert_eq!(layer.layer_type, LayerType::ObjectGroup);
        assert_eq!(layer.objects.len(), 3);

        // Platform object
        let platform = &layer.objects[0];
        assert_eq!(platform.id, 2);
        assert_eq!(platform.object_type, MapObjectType::Platform);
        assert_eq!(platform.x, 0.0);
        assert_eq!(platform.y, 672.0);
        assert_eq!(platform.width, 416.0);
        assert_eq!(platform.height, 64.0);
        assert_eq!(platform.center(), Vector2::new(208.0, 704.0));

        // Exit object
        let exit = &layer.objects[1];
        assert_eq!(exit.id, 17);
        assert_eq!(exit.object_type, MapObjectType::Exit);
        assert_eq!(exit.center(), Vector2::new(4320.0 + 80.0, 192.0 + 32.0));

        // Spawn object (point)
        let spawn = &layer.objects[2];
        assert_eq!(spawn.id, 21);
        assert_eq!(spawn.object_type, MapObjectType::Spawn);
        assert_eq!(spawn.center(), Vector2::new(160.0, 544.0));
        assert_eq!(map.spawn_point(), Some(Vector2::new(160.0, 544.0)));
    }

    #[test]
    fn test_load_act_two_map_file() {
        let map = Map::load_from_file("resources/maps/act_two/map1.json")
            .expect("Should load act_two/map1.json");

        assert_eq!(map.width, 140);
        assert_eq!(map.height, 35);
        assert_eq!(map.pixel_width(), 4480.0);
        assert_eq!(map.pixel_height(), 1120.0);

        let platform_count = map.platforms().count();
        assert_eq!(platform_count, 18, "Expected 18 platforms in map1");

        let exit_count = map.exits().count();
        assert_eq!(exit_count, 1, "Expected 1 Exit object");

        let gobbler_count = map.gobblers().count();
        assert_eq!(gobbler_count, 3, "Expected 3 Gobbler enemy spawn objects in map1");

        let spawn = map.spawn_point();
        assert_eq!(spawn, Some(Vector2::new(96.0, 912.0)));
    }

    #[test]
    fn test_map_spawns_into_world() {
        let map = Map::from_json_str(SAMPLE_TILED_JSON).unwrap();
        let mut world = World::new();

        let result = map.spawn_into_world(&mut world);
        assert_eq!(result.platforms.len(), 1);
        assert_eq!(result.exits.len(), 1);
        assert_eq!(result.spawn_point, Some(Vector2::new(160.0, 544.0)));

        let plat_id = result.platforms[0];
        assert_eq!(world.tags[plat_id.0], EntityType::Ground);
        assert!(world.alive[plat_id.0]);
        let plat_pos = world.positions[plat_id.0];
        assert_eq!(plat_pos, Vector2::new(208.0, 704.0));

        let plat_col = world.colliders[plat_id.0].unwrap();
        assert_eq!(plat_col.width, 416.0);
        assert_eq!(plat_col.height, 64.0);
        assert!(plat_col.is_solid);

        let plat_phys = world.physics_data[plat_id.0].as_ref().unwrap();
        assert!(plat_phys.is_static);

        let exit_id = result.exits[0];
        assert_eq!(world.tags[exit_id.0], EntityType::Obstacle);
        let exit_col = world.colliders[exit_id.0].unwrap();
        assert!(!exit_col.is_solid, "Exit should be a non-solid sensor");
    }

    #[test]
    fn test_player_lands_on_spawned_map_platform() {
        let map = Map::from_json_str(SAMPLE_TILED_JSON).unwrap();
        let mut world = World::new();
        let result = map.spawn_into_world(&mut world);

        // Spawn player above the first platform at x = 160, y = 544 (spawn marker)
        let player_w = 20.0;
        let player_h = 20.0;
        let spawn_pos = result.spawn_point.unwrap();

        let player_id = world
            .spawn(
                EntityType::Player,
                spawn_pos,
                Vector2::new(0.0, 0.0),
                0.0,
                true,
                SpriteID::Player,
            )
            .with_physics_preset(crate::components::PhysicsPreset::Normal)
            .with_collider(Collider::new(player_w, player_h))
            .build();

        // Platform top is y = 672. Player resting center should be 672 - 10 = 662.
        for _ in 0..120 {
            crate::systems::physics::update(&mut world, 1.0 / 60.0);
            crate::systems::collision::update(&mut world, 1.0 / 60.0);
        }

        let landed_pos = world.positions[player_id.0];
        assert!(
            (landed_pos.y - 662.0).abs() < 1e-2,
            "Player should land on top of platform at y = 662.0, got {}",
            landed_pos.y
        );
        assert_eq!(world.velocities[player_id.0].y, 0.0);
    }

    #[test]
    fn test_parse_utf8_strings() {
        let json = r#"{"name": "Gegner üö äß", "escape": "hello\nworld \u00e4\u00f6", "emoji": "🚀"}"#;
        let mut parser = JsonParser::new(json);
        let val = parser.parse_value().expect("Should parse valid UTF-8 JSON");
        assert_eq!(val.get("name").and_then(|v| v.as_str()), Some("Gegner üö äß"));
        assert_eq!(val.get("escape").and_then(|v| v.as_str()), Some("hello\nworld äö"));
        assert_eq!(val.get("emoji").and_then(|v| v.as_str()), Some("🚀"));
    }

    #[test]
    fn test_empty_type_falls_back_to_class() {
        let json = r#"{
            "width": 10, "height": 10, "tilewidth": 16, "tileheight": 16,
            "layers": [{
                "id": 1, "name": "Obj", "type": "objectgroup", "visible": true, "opacity": 1.0,
                "objects": [
                    { "id": 1, "name": "p1", "type": "", "class": "Platform", "x": 10, "y": 20, "width": 30, "height": 10 },
                    { "id": 2, "name": "e1", "type": "", "class": "Exit", "x": 50, "y": 20, "width": 10, "height": 10 },
                    { "id": 3, "name": "s1", "type": "", "class": "Spawn", "x": 5, "y": 5, "width": 0, "height": 0 }
                ]
            }]
        }"#;
        let map = Map::from_json_str(json).expect("Should parse map with empty type and filled class");
        assert_eq!(map.platforms().count(), 1);
        assert_eq!(map.exits().count(), 1);
        assert_eq!(map.spawn_point(), Some(Vector2::new(5.0, 5.0)));
    }

    #[test]
    fn test_rotated_object_center() {
        let obj_90 = MapObject {
            id: 1,
            name: "rotated".to_string(),
            object_type: MapObjectType::Platform,
            x: 100.0,
            y: 100.0,
            width: 20.0,
            height: 10.0,
            rotation: 90.0,
            visible: true,
        };
        let c = obj_90.center();
        assert!((c.x - 95.0).abs() < 1e-4, "Expected center.x ~ 95.0, got {}", c.x);
        assert!((c.y - 110.0).abs() < 1e-4, "Expected center.y ~ 110.0, got {}", c.y);

        let obj_0 = MapObject {
            id: 2,
            name: "unrotated".to_string(),
            object_type: MapObjectType::Platform,
            x: 100.0,
            y: 100.0,
            width: 20.0,
            height: 10.0,
            rotation: 0.0,
            visible: true,
        };
        assert_eq!(obj_0.center(), Vector2::new(110.0, 105.0));
    }

    #[test]
    fn test_act_two_map_tiles_layer_and_tileset() {
        let map = Map::load_from_file("resources/maps/act_two/map1.json")
            .expect("Should load act_two/map1.json");

        // Verify tileset metadata
        assert_eq!(map.tilesets().len(), 1);
        let ts = &map.tilesets()[0];
        assert_eq!(ts.name, "Platforms");
        assert_eq!(ts.first_gid, 1);
        assert_eq!(ts.columns, 16);
        assert_eq!(ts.tile_width, 32);
        assert_eq!(ts.tile_height, 32);
        assert_eq!(ts.tile_count, 256);
        assert_eq!(ts.image_width, 512);
        assert_eq!(ts.image_height, 512);
        assert_eq!(ts.image, "resources/graphics/Platforms.png");
        assert!(
            std::path::Path::new(&ts.image).exists(),
            "Tileset image file '{}' must exist on disk",
            ts.image
        );

        // Verify tile layer decompression
        let tile_layers: Vec<&MapLayer> = map.tile_layers().collect();
        assert_eq!(tile_layers.len(), 1);
        let tiles_layer = tile_layers[0];
        assert_eq!(tiles_layer.name, "Tiles");
        assert_eq!(tiles_layer.width, 140);
        assert_eq!(tiles_layer.height, 35);
        assert_eq!(tiles_layer.data.len(), 140 * 35);

        let non_empty: Vec<(u32, u32, u32)> = tiles_layer.non_empty_tiles().collect();
        assert_eq!(
            non_empty,
            vec![
                (2, 32, 1),
                (3, 32, 2),
                (4, 32, 3),
                (5, 32, 4),
                (6, 32, 5),
                (2, 33, 18),
                (3, 33, 19),
                (4, 33, 20),
                (5, 33, 21),
                (2, 34, 129),
                (3, 34, 130),
                (4, 34, 131),
                (5, 34, 132),
            ]
        );

        // Verify source rectangle calculations
        assert_eq!(
            map.tile_source_rect(1),
            Some(Rectangle::new(0.0, 0.0, 32.0, 32.0))
        );
        assert_eq!(
            map.tile_source_rect(5),
            Some(Rectangle::new(128.0, 0.0, 32.0, 32.0))
        );
        assert_eq!(
            map.tile_source_rect(18),
            Some(Rectangle::new(32.0, 32.0, 32.0, 32.0))
        );
        assert_eq!(
            map.tile_source_rect(129),
            Some(Rectangle::new(0.0, 256.0, 32.0, 32.0))
        );
        assert_eq!(map.tile_source_rect(0), None);
    }
}
