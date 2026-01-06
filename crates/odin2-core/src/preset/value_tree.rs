//! JUCE ValueTree Binary Parser for Odin 2 Presets
//!
//! This is a specialized parser for the Odin 2 .odin preset format.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::io::{self, Read, Seek};

/// Error type for ValueTree parsing
#[derive(Debug)]
pub enum ValueTreeError {
    Io(io::Error),
    InvalidFormat(String),
    UnexpectedEof,
    InvalidUtf8,
    UnsupportedType(u8),
}

impl From<io::Error> for ValueTreeError {
    fn from(e: io::Error) -> Self {
        ValueTreeError::Io(e)
    }
}

impl std::fmt::Display for ValueTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueTreeError::Io(e) => write!(f, "IO error: {}", e),
            ValueTreeError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            ValueTreeError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ValueTreeError::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            ValueTreeError::UnsupportedType(t) => write!(f, "Unsupported value type: {}", t),
        }
    }
}

impl std::error::Error for ValueTreeError {}

/// Variant value type
#[derive(Debug, Clone)]
pub enum VariantValue {
    Int(i32),
    Int64(i64),
    Double(f64),
    String(String),
    Bool(bool),
}

impl VariantValue {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            VariantValue::Double(d) => Some(*d as f32),
            VariantValue::Int(i) => Some(*i as f32),
            VariantValue::Int64(i) => Some(*i as f32),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            VariantValue::Double(d) => Some(*d),
            VariantValue::Int(i) => Some(*i as f64),
            VariantValue::Int64(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            VariantValue::Int(i) => Some(*i),
            VariantValue::Int64(i) => Some(*i as i32),
            VariantValue::Double(d) => Some(*d as i32),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            VariantValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            VariantValue::Bool(b) => Some(*b),
            VariantValue::Int(i) => Some(*i != 0),
            VariantValue::Double(d) => Some(*d != 0.0),
            _ => None,
        }
    }
}

/// Parsed Odin preset data
#[derive(Debug, Clone, Default)]
pub struct ValueTree {
    /// Audio parameters (PARAM entries)
    pub params: BTreeMap<String, f64>,
    /// FX section properties
    pub fx: BTreeMap<String, VariantValue>,
    /// LFO section properties
    pub lfo: BTreeMap<String, VariantValue>,
    /// Misc section properties
    pub misc: BTreeMap<String, VariantValue>,
    /// Mod matrix section properties
    pub mod_matrix: BTreeMap<String, VariantValue>,
    /// Oscillator section properties
    pub osc: BTreeMap<String, VariantValue>,
}

impl ValueTree {
    /// Parse an Odin preset from binary data
    pub fn from_bytes(data: &[u8]) -> Result<Self, ValueTreeError> {
        let mut cursor = io::Cursor::new(data);
        Self::read_from(&mut cursor)
    }

    /// Read an Odin preset from a reader
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self, ValueTreeError> {
        let mut tree = ValueTree::default();

        // Read the entire file into memory for easier parsing
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        // Verify header
        if data.len() < 5 || &data[0..4] != b"Odin" {
            return Err(ValueTreeError::InvalidFormat(
                "Not an Odin preset file".to_string(),
            ));
        }

        // Parse PARAM entries
        tree.parse_params(&data)?;

        // Parse section properties
        tree.parse_section(&data, b"fx\x00", &mut tree.fx.clone())?;
        tree.parse_section(&data, b"lfo\x00", &mut tree.lfo.clone())?;
        tree.parse_section(&data, b"misc\x00", &mut tree.misc.clone())?;
        tree.parse_section(&data, b"osc\x00", &mut tree.osc.clone())?;

        // Re-parse to actually store values (avoiding borrow issues)
        let mut fx = BTreeMap::new();
        let mut lfo = BTreeMap::new();
        let mut misc = BTreeMap::new();
        let mut osc = BTreeMap::new();
        let mut mod_matrix = BTreeMap::new();

        Self::parse_section_into(&data, b"fx\x00", &mut fx);
        Self::parse_section_into(&data, b"lfo\x00", &mut lfo);
        Self::parse_section_into(&data, b"misc\x00", &mut misc);
        Self::parse_section_into(&data, b"osc\x00", &mut osc);
        Self::parse_section_into(&data, b"mod\x00", &mut mod_matrix);

        tree.fx = fx;
        tree.lfo = lfo;
        tree.misc = misc;
        tree.osc = osc;
        tree.mod_matrix = mod_matrix;

        Ok(tree)
    }

    fn parse_params(&mut self, data: &[u8]) -> Result<(), ValueTreeError> {
        let param_marker = b"PARAM\x00";
        let mut pos = 0;

        while let Some(idx) = find_bytes(&data[pos..], param_marker) {
            let abs_idx = pos + idx;
            pos = abs_idx + param_marker.len();

            // Skip header bytes (01 02)
            if pos + 2 > data.len() {
                break;
            }
            pos += 2;

            // Read "id" property name
            let (id_name, new_pos) = read_string(data, pos)?;
            pos = new_pos;

            if id_name != "id" {
                continue;
            }

            // Skip: 01 length
            if pos + 2 > data.len() {
                break;
            }
            pos += 2;

            // Read type byte
            let type_byte = data[pos];
            pos += 1;

            // Read parameter ID (string)
            if type_byte != 5 {
                continue;
            }
            let (param_id, new_pos) = read_string(data, pos)?;
            pos = new_pos;

            // Read "value" property name
            let (value_name, new_pos) = read_string(data, pos)?;
            pos = new_pos;

            if value_name != "value" {
                continue;
            }

            // Skip: 01 length
            if pos + 2 > data.len() {
                break;
            }
            pos += 2;

            // Read type byte
            let type_byte = data[pos];
            pos += 1;

            // Read value (should be double)
            if (type_byte == 4 || type_byte == 9) && pos + 8 <= data.len() {
                let value = f64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
                self.params.insert(param_id, value);
                pos += 8;
            }
        }

        Ok(())
    }

    fn parse_section(
        &self,
        data: &[u8],
        marker: &[u8],
        _props: &mut BTreeMap<String, VariantValue>,
    ) -> Result<(), ValueTreeError> {
        // Find section marker
        if find_bytes(data, marker).is_none() {
            return Ok(());
        }
        Ok(())
    }

    fn parse_section_into(
        data: &[u8],
        marker: &[u8],
        props: &mut BTreeMap<String, VariantValue>,
    ) {
        if let Some(idx) = find_bytes(data, marker) {
            let mut pos = idx + marker.len();

            // Read number of properties (01 + count as single byte after that)
            if pos >= data.len() {
                return;
            }

            // Skip the 01 byte if present
            if data[pos] == 0x01 {
                pos += 1;
            }

            // Read property count (next byte)
            if pos >= data.len() {
                return;
            }
            let num_props = data[pos] as usize;
            pos += 1;

            // Read properties
            for _ in 0..num_props {
                if pos >= data.len() {
                    break;
                }

                // Read property name
                let (prop_name, new_pos) = match read_string(data, pos) {
                    Ok(r) => r,
                    Err(_) => break,
                };
                pos = new_pos;

                // Skip 01 length bytes
                if pos + 2 > data.len() {
                    break;
                }
                pos += 2;

                // Read type byte
                if pos >= data.len() {
                    break;
                }
                let type_byte = data[pos];
                pos += 1;

                // Read value based on type
                let value = match type_byte {
                    1 => {
                        // Int32
                        if pos + 4 > data.len() {
                            break;
                        }
                        let v = i32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
                        pos += 4;
                        VariantValue::Int(v)
                    }
                    2 => VariantValue::Bool(false),
                    3 => VariantValue::Bool(true),
                    4 | 9 => {
                        // Double
                        if pos + 8 > data.len() {
                            break;
                        }
                        let v = f64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
                        pos += 8;
                        VariantValue::Double(v)
                    }
                    5 => {
                        // String
                        let (s, new_pos) = match read_string(data, pos) {
                            Ok(r) => r,
                            Err(_) => break,
                        };
                        pos = new_pos;
                        VariantValue::String(s)
                    }
                    _ => {
                        // Unknown type, try to skip
                        continue;
                    }
                };

                props.insert(prop_name, value);
            }
        }
    }

    /// Get a parameter value
    pub fn get_param(&self, name: &str) -> Option<f64> {
        self.params.get(name).copied()
    }

    /// Get an FX property
    pub fn get_fx(&self, name: &str) -> Option<&VariantValue> {
        self.fx.get(name)
    }

    /// Get an LFO property
    pub fn get_lfo(&self, name: &str) -> Option<&VariantValue> {
        self.lfo.get(name)
    }

    /// Get a misc property
    pub fn get_misc(&self, name: &str) -> Option<&VariantValue> {
        self.misc.get(name)
    }

    /// Get an osc property
    pub fn get_osc(&self, name: &str) -> Option<&VariantValue> {
        self.osc.get(name)
    }

    /// Get a mod matrix property
    pub fn get_mod(&self, name: &str) -> Option<&VariantValue> {
        self.mod_matrix.get(name)
    }
}

/// Find a byte pattern in data
fn find_bytes(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}

/// Read a null-terminated string
fn read_string(data: &[u8], offset: usize) -> Result<(String, usize), ValueTreeError> {
    if offset >= data.len() {
        return Err(ValueTreeError::UnexpectedEof);
    }

    let end = data[offset..]
        .iter()
        .position(|&b| b == 0)
        .ok_or(ValueTreeError::UnexpectedEof)?;

    let s = String::from_utf8(data[offset..offset + end].to_vec())
        .map_err(|_| ValueTreeError::InvalidUtf8)?;

    Ok((s, offset + end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::println;

    #[test]
    fn test_parse_preset() {
        let path = "/Users/william.quintal/Project/bascanada/odin2-rs/odin2/assets/Soundbanks/Factory Presets/Bass/Analog Bass [tx].odin";
        if std::path::Path::new(path).exists() {
            let data = std::fs::read(path).unwrap();
            let result = ValueTree::from_bytes(&data);
            match result {
                Ok(tree) => {
                    println!("Parsed {} parameters", tree.params.len());
                    println!("Parsed {} osc properties", tree.osc.len());
                    println!("Parsed {} misc properties", tree.misc.len());

                    if let Some(v) = tree.get_param("osc1_vol") {
                        println!("osc1_vol = {}", v);
                    }
                    if let Some(v) = tree.get_param("fil1_freq") {
                        println!("fil1_freq = {}", v);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}
