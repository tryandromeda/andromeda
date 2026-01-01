// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use cosmic_text::{FontSystem, Style, Weight, fontdb};

/// Font descriptor matching Canvas2D font properties
#[derive(Debug, Clone)]
pub struct FontDescriptor {
    pub family: String,
    pub size: f32,
    pub weight: u16,
    pub style: Style,
}

// Manual implementations for hash/eq to handle f32
impl PartialEq for FontDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.family == other.family
            && self.size.to_bits() == other.size.to_bits()
            && self.weight == other.weight
            && self.style == other.style
    }
}

impl Eq for FontDescriptor {}

impl std::hash::Hash for FontDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.family.hash(state);
        self.size.to_bits().hash(state);
        self.weight.hash(state);
        self.style.hash(state);
    }
}

impl Default for FontDescriptor {
    fn default() -> Self {
        Self {
            family: "sans-serif".to_string(),
            size: 10.0,
            weight: 400,
            style: Style::Normal,
        }
    }
}

/// Font manager for loading system fonts and parsing CSS font strings
pub struct FontManager {
    pub font_system: FontSystem,
}

impl FontManager {
    /// Create a new font manager with system fonts loaded
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let font_system = FontSystem::new_with_fonts([]);

        Self { font_system }
    }

    /// Parse a CSS font string into a FontDescriptor
    pub fn parse_font_string(font_string: &str) -> Result<FontDescriptor, String> {
        let mut descriptor = FontDescriptor::default();
        let parts: Vec<&str> = font_string.split_whitespace().collect();

        if parts.is_empty() {
            return Err("Empty font string".to_string());
        }

        let mut idx = 0;

        if idx < parts.len() {
            match parts[idx] {
                "italic" | "oblique" => {
                    descriptor.style = Style::Italic;
                    idx += 1;
                }
                "normal" => {
                    descriptor.style = Style::Normal;
                    idx += 1;
                }
                _ => {}
            }
        }

        if idx < parts.len() {
            match parts[idx] {
                "bold" => {
                    descriptor.weight = 700;
                    idx += 1;
                }
                "normal" => {
                    descriptor.weight = 400;
                    idx += 1;
                }
                "lighter" => {
                    descriptor.weight = 300;
                    idx += 1;
                }
                "bolder" => {
                    descriptor.weight = 700;
                    idx += 1;
                }
                weight_str => {
                    if let Ok(weight) = weight_str.parse::<u16>()
                        && (100..=900).contains(&weight)
                    {
                        descriptor.weight = weight;
                        idx += 1;
                    }
                }
            }
        }

        if idx >= parts.len() {
            return Err("Missing font size".to_string());
        }

        let size_str = parts[idx];
        if let Some(size_num) = size_str.strip_suffix("px") {
            descriptor.size = size_num
                .parse::<f32>()
                .map_err(|_| format!("Invalid font size: {}", size_str))?;
            idx += 1;
        } else {
            return Err(format!("Font size must have 'px' suffix: {}", size_str));
        }
        if idx >= parts.len() {
            return Err("Missing font family".to_string());
        }

        let family_parts = &parts[idx..];
        let family = family_parts.join(" ");

        let family = family
            .split(',')
            .next()
            .unwrap_or("sans-serif")
            .trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .to_string();

        descriptor.family = if family.is_empty() {
            "sans-serif".to_string()
        } else {
            family
        };

        Ok(descriptor)
    }

    /// Get or load a specific font by descriptor
    pub fn ensure_font(&mut self, descriptor: &FontDescriptor) -> bool {
        let db = self.font_system.db();
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(&descriptor.family)],
            weight: Weight(descriptor.weight),
            style: descriptor.style,
            ..Default::default()
        };

        db.query(&query).is_some()
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}
