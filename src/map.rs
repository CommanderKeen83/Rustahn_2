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
    pub objects: Vec<MapObject>,
}

/// Level map loaded from a Tiled JSON map export.
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<MapLayer>,
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
        Self::from_json_str(&content)
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
                    objects,
                });
            }
        }

        Ok(Self {
            width,
            height,
            tile_width,
            tile_height,
            layers,
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
}
