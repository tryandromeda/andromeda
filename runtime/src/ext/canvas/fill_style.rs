// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Represents different fill styles for Canvas 2D operations
#[derive(Clone, Debug)]
pub enum FillStyle {
    /// Solid color specified as RGBA values (0.0-1.0)
    Color { r: f32, g: f32, b: f32, a: f32 },
    /// Linear gradient (placeholder for future implementation)
    LinearGradient,
    /// Radial gradient (placeholder for future implementation)
    RadialGradient,
    /// Pattern (placeholder for future implementation)
    Pattern,
}

impl Default for FillStyle {
    fn default() -> Self {
        // Default to black color
        FillStyle::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl FillStyle {
    /// Parse a CSS color string into a FillStyle
    pub fn from_css_color(color_str: &str) -> Result<Self, String> {
        let color_str = color_str.trim();

        // Handle hex colors
        if color_str.starts_with('#') {
            return Self::parse_hex_color(color_str);
        }

        // Handle rgb() and rgba() colors
        if color_str.starts_with("rgb(") || color_str.starts_with("rgba(") {
            return Self::parse_rgb_color(color_str);
        }

        // Handle named colors
        Self::parse_named_color(color_str)
    }

    fn parse_hex_color(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            3 => {
                // Short hex format like #RGB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                Ok(FillStyle::Color {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: 1.0,
                })
            }
            6 => {
                // Full hex format like #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                Ok(FillStyle::Color {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: 1.0,
                })
            }
            8 => {
                // Hex with alpha like #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex color")?;
                Ok(FillStyle::Color {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: a as f32 / 255.0,
                })
            }
            _ => Err("Invalid hex color length".to_string()),
        }
    }

    fn parse_rgb_color(rgb: &str) -> Result<Self, String> {
        let is_rgba = rgb.starts_with("rgba(");
        let inner = if is_rgba {
            rgb.trim_start_matches("rgba(").trim_end_matches(')')
        } else {
            rgb.trim_start_matches("rgb(").trim_end_matches(')')
        };

        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        if (!is_rgba && parts.len() != 3) || (is_rgba && parts.len() != 4) {
            return Err("Invalid rgb/rgba format".to_string());
        }

        let r = parts[0].parse::<f32>().map_err(|_| "Invalid red value")? / 255.0;
        let g = parts[1].parse::<f32>().map_err(|_| "Invalid green value")? / 255.0;
        let b = parts[2].parse::<f32>().map_err(|_| "Invalid blue value")? / 255.0;
        let a = if is_rgba {
            parts[3].parse::<f32>().map_err(|_| "Invalid alpha value")?
        } else {
            1.0
        };

        Ok(FillStyle::Color { r, g, b, a })
    }

    fn parse_named_color(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "aliceblue" => Ok(FillStyle::Color {
                r: 0.941,
                g: 0.973,
                b: 1.0,
                a: 1.0,
            }),
            "antiquewhite" => Ok(FillStyle::Color {
                r: 0.98,
                g: 0.92,
                b: 0.84,
                a: 1.0,
            }),
            "aqua" => Ok(FillStyle::Color {
                r: 0.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            "aquamarine" => Ok(FillStyle::Color {
                r: 0.498,
                g: 1.0,
                b: 0.831,
                a: 1.0,
            }),
            "azure" => Ok(FillStyle::Color {
                r: 0.941,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            "beige" => Ok(FillStyle::Color {
                r: 0.96,
                g: 0.96,
                b: 0.86,
                a: 1.0,
            }),
            "bisque" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.894,
                b: 0.769,
                a: 1.0,
            }),
            "black" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            "blanchedalmond" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.922,
                b: 0.804,
                a: 1.0,
            }),
            "blue" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }),
            "blueviolet" => Ok(FillStyle::Color {
                r: 0.541,
                g: 0.169,
                b: 0.886,
                a: 1.0,
            }),
            "brown" => Ok(FillStyle::Color {
                r: 0.647,
                g: 0.165,
                b: 0.165,
                a: 1.0,
            }),
            "burlywood" => Ok(FillStyle::Color {
                r: 0.871,
                g: 0.722,
                b: 0.529,
                a: 1.0,
            }),
            "cadetblue" => Ok(FillStyle::Color {
                r: 0.373,
                g: 0.62,
                b: 0.627,
                a: 1.0,
            }),
            "cameo" => Ok(FillStyle::Color {
                r: 0.937,
                g: 0.867,
                b: 0.702,
                a: 1.0,
            }),
            "chartreuse" => Ok(FillStyle::Color {
                r: 0.498,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            "chocolate" => Ok(FillStyle::Color {
                r: 0.824,
                g: 0.412,
                b: 0.118,
                a: 1.0,
            }),
            "coral" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.498,
                b: 0.314,
                a: 1.0,
            }),
            "cornflowerblue" => Ok(FillStyle::Color {
                r: 0.392,
                g: 0.584,
                b: 0.929,
                a: 1.0,
            }),
            "crimson" => Ok(FillStyle::Color {
                r: 0.863,
                g: 0.078,
                b: 0.235,
                a: 1.0,
            }),
            "cyan" => Ok(FillStyle::Color {
                r: 0.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            "darkblue" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.545,
                a: 1.0,
            }),
            "darkcyan" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.545,
                b: 0.545,
                a: 1.0,
            }),
            "darkgoldenrod" => Ok(FillStyle::Color {
                r: 0.722,
                g: 0.525,
                b: 0.043,
                a: 1.0,
            }),
            "darkgray" => Ok(FillStyle::Color {
                r: 0.663,
                g: 0.663,
                b: 0.663,
                a: 1.0,
            }),
            "darkgreen" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.392,
                b: 0.0,
                a: 1.0,
            }),
            "darkkhaki" => Ok(FillStyle::Color {
                r: 0.741,
                g: 0.717,
                b: 0.419,
                a: 1.0,
            }),
            "darkmagenta" => Ok(FillStyle::Color {
                r: 0.545,
                g: 0.0,
                b: 0.545,
                a: 1.0,
            }),
            "darkolivegreen" => Ok(FillStyle::Color {
                r: 0.333,
                g: 0.42,
                b: 0.184,
                a: 1.0,
            }),
            "darkorange" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.549,
                b: 0.0,
                a: 1.0,
            }),
            "darkorchid" => Ok(FillStyle::Color {
                r: 0.6,
                g: 0.196,
                b: 0.8,
                a: 1.0,
            }),
            "darkred" => Ok(FillStyle::Color {
                r: 0.545,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            "darksalmon" => Ok(FillStyle::Color {
                r: 0.914,
                g: 0.588,
                b: 0.478,
                a: 1.0,
            }),
            "darkseagreen" => Ok(FillStyle::Color {
                r: 0.561,
                g: 0.737,
                b: 0.561,
                a: 1.0,
            }),
            "darkslateblue" => Ok(FillStyle::Color {
                r: 0.282,
                g: 0.239,
                b: 0.545,
                a: 1.0,
            }),
            "darkslategray" => Ok(FillStyle::Color {
                r: 0.184,
                g: 0.31,
                b: 0.31,
                a: 1.0,
            }),
            "darkturquoise" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.807,
                b: 0.819,
                a: 1.0,
            }),
            "darkviolet" => Ok(FillStyle::Color {
                r: 0.58,
                g: 0.0,
                b: 0.827,
                a: 1.0,
            }),
            "deeppink" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.078,
                b: 0.576,
                a: 1.0,
            }),
            "deepskyblue" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.749,
                b: 1.0,
                a: 1.0,
            }),
            "dimgray" => Ok(FillStyle::Color {
                r: 0.412,
                g: 0.412,
                b: 0.412,
                a: 1.0,
            }),
            "dimpurple" => Ok(FillStyle::Color {
                r: 0.415,
                g: 0.313,
                b: 0.494,
                a: 1.0,
            }),
            "dodgerblue" => Ok(FillStyle::Color {
                r: 0.118,
                g: 0.565,
                b: 1.0,
                a: 1.0,
            }),
            "firebrick" => Ok(FillStyle::Color {
                r: 0.698,
                g: 0.132,
                b: 0.203,
                a: 1.0,
            }),
            "floralwhite" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.98,
                b: 0.941,
                a: 1.0,
            }),
            "forestgreen" => Ok(FillStyle::Color {
                r: 0.133,
                g: 0.545,
                b: 0.133,
                a: 1.0,
            }),
            "fuchsia" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }),
            "gainsboro" => Ok(FillStyle::Color {
                r: 0.863,
                g: 0.863,
                b: 0.863,
                a: 1.0,
            }),
            "ghostwhite" => Ok(FillStyle::Color {
                r: 0.973,
                g: 0.973,
                b: 1.0,
                a: 1.0,
            }),
            "gold" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.843,
                b: 0.0,
                a: 1.0,
            }),
            "goldenrod" => Ok(FillStyle::Color {
                r: 0.855,
                g: 0.647,
                b: 0.125,
                a: 1.0,
            }),
            "gray" => Ok(FillStyle::Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            }),
            "green" => Ok(FillStyle::Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            "greenyellow" => Ok(FillStyle::Color {
                r: 0.678,
                g: 1.0,
                b: 0.184,
                a: 1.0,
            }),
            "honeydew" => Ok(FillStyle::Color {
                r: 0.941,
                g: 1.0,
                b: 0.941,
                a: 1.0,
            }),
            "hotpink" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.412,
                b: 0.706,
                a: 1.0,
            }),
            "indianred" => Ok(FillStyle::Color {
                r: 0.804,
                g: 0.361,
                b: 0.361,
                a: 1.0,
            }),
            "indigo" => Ok(FillStyle::Color {
                r: 0.294,
                g: 0.0,
                b: 0.51,
                a: 1.0,
            }),
            "ivory" => Ok(FillStyle::Color {
                r: 1.0,
                g: 1.0,
                b: 0.941,
                a: 1.0,
            }),
            "khaki" => Ok(FillStyle::Color {
                r: 0.941,
                g: 0.902,
                b: 0.549,
                a: 1.0,
            }),
            "lavender" => Ok(FillStyle::Color {
                r: 0.902,
                g: 0.902,
                b: 0.98,
                a: 1.0,
            }),
            "lavenderblush" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.941,
                b: 0.961,
                a: 1.0,
            }),
            "lawngreen" => Ok(FillStyle::Color {
                r: 0.486,
                g: 0.988,
                b: 0.0,
                a: 1.0,
            }),
            "lemonchiffon" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.98,
                b: 0.804,
                a: 1.0,
            }),
            "lightblue" => Ok(FillStyle::Color {
                r: 0.678,
                g: 0.847,
                b: 0.902,
                a: 1.0,
            }),
            "lightcoral" => Ok(FillStyle::Color {
                r: 0.941,
                g: 0.502,
                b: 0.502,
                a: 1.0,
            }),
            "lightcyan" => Ok(FillStyle::Color {
                r: 0.878,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            "lightgoldenrodyellow" => Ok(FillStyle::Color {
                r: 0.980,
                g: 0.980,
                b: 0.824,
                a: 1.0,
            }),
            "lightgray" => Ok(FillStyle::Color {
                r: 0.827,
                g: 0.827,
                b: 0.827,
                a: 1.0,
            }),
            "lightgreen" => Ok(FillStyle::Color {
                r: 0.565,
                g: 0.933,
                b: 0.565,
                a: 1.0,
            }),
            "lightpink" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.714,
                b: 0.757,
                a: 1.0,
            }),
            "lightsalmon" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.627,
                b: 0.478,
                a: 1.0,
            }),
            "lightseagreen" => Ok(FillStyle::Color {
                r: 0.125,
                g: 0.698,
                b: 0.667,
                a: 1.0,
            }),
            "lightskyblue" => Ok(FillStyle::Color {
                r: 0.529,
                g: 0.808,
                b: 0.98,
                a: 1.0,
            }),
            "lightslategray" => Ok(FillStyle::Color {
                r: 0.467,
                g: 0.533,
                b: 0.6,
                a: 1.0,
            }),
            "lightsteelblue" => Ok(FillStyle::Color {
                r: 0.690,
                g: 0.769,
                b: 0.871,
                a: 1.0,
            }),
            "lightyellow" => Ok(FillStyle::Color {
                r: 1.0,
                g: 1.0,
                b: 0.878,
                a: 1.0,
            }),
            "lime" => Ok(FillStyle::Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            "limegreen" => Ok(FillStyle::Color {
                r: 0.196,
                g: 0.804,
                b: 0.196,
                a: 1.0,
            }),
            "linen" => Ok(FillStyle::Color {
                r: 0.98,
                g: 0.941,
                b: 0.902,
                a: 1.0,
            }),
            "magenta" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }),
            "maroon" => Ok(FillStyle::Color {
                r: 0.5,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            "mediumaquamarine" => Ok(FillStyle::Color {
                r: 0.4,
                g: 0.804,
                b: 0.667,
                a: 1.0,
            }),
            "mediumblue" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.804,
                a: 1.0,
            }),
            "mediumorchid" => Ok(FillStyle::Color {
                r: 0.729,
                g: 0.333,
                b: 0.827,
                a: 1.0,
            }),
            "mediumpurple" => Ok(FillStyle::Color {
                r: 0.576,
                g: 0.439,
                b: 0.859,
                a: 1.0,
            }),
            "mediumseagreen" => Ok(FillStyle::Color {
                r: 0.235,
                g: 0.702,
                b: 0.443,
                a: 1.0,
            }),
            "mediumslateblue" => Ok(FillStyle::Color {
                r: 0.482,
                g: 0.408,
                b: 0.933,
                a: 1.0,
            }),
            "mediumspringgreen" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.98,
                b: 0.604,
                a: 1.0,
            }),
            "mediumturquoise" => Ok(FillStyle::Color {
                r: 0.282,
                g: 0.82,
                b: 0.8,
                a: 1.0,
            }),
            "mediumvioletred" => Ok(FillStyle::Color {
                r: 0.78,
                g: 0.082,
                b: 0.522,
                a: 1.0,
            }),
            "midnightblue" => Ok(FillStyle::Color {
                r: 0.098,
                g: 0.098,
                b: 0.439,
                a: 1.0,
            }),
            "mintcream" => Ok(FillStyle::Color {
                r: 0.961,
                g: 1.0,
                b: 0.98,
                a: 1.0,
            }),
            "mistyrose" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.894,
                b: 0.882,
                a: 1.0,
            }),
            "mocha" => Ok(FillStyle::Color {
                r: 0.824,
                g: 0.706,
                b: 0.549,
                a: 1.0,
            }),
            "navajowhite" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.871,
                b: 0.678,
                a: 1.0,
            }),
            "navy" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.0,
                b: 0.5,
                a: 1.0,
            }),
            "oldlace" => Ok(FillStyle::Color {
                r: 0.992,
                g: 0.961,
                b: 0.902,
                a: 1.0,
            }),
            "olive" => Ok(FillStyle::Color {
                r: 0.5,
                g: 0.5,
                b: 0.0,
                a: 1.0,
            }),
            "olivedrab" => Ok(FillStyle::Color {
                r: 0.42,
                g: 0.557,
                b: 0.137,
                a: 1.0,
            }),
            "orange" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.647,
                b: 0.0,
                a: 1.0,
            }),
            "orangered" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.271,
                b: 0.0,
                a: 1.0,
            }),
            "orchid" => Ok(FillStyle::Color {
                r: 0.855,
                g: 0.439,
                b: 0.839,
                a: 1.0,
            }),
            "palegoldenrod" => Ok(FillStyle::Color {
                r: 0.933,
                g: 0.91,
                b: 0.667,
                a: 1.0,
            }),
            "palegreen" => Ok(FillStyle::Color {
                r: 0.596,
                g: 0.984,
                b: 0.596,
                a: 1.0,
            }),
            "paleturquoise" => Ok(FillStyle::Color {
                r: 0.686,
                g: 0.933,
                b: 0.933,
                a: 1.0,
            }),
            "palevioletred" => Ok(FillStyle::Color {
                r: 0.859,
                g: 0.439,
                b: 0.576,
                a: 1.0,
            }),
            "papayawhip" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.937,
                b: 0.835,
                a: 1.0,
            }),
            "peachpuff" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.855,
                b: 0.725,
                a: 1.0,
            }),
            "peru" => Ok(FillStyle::Color {
                r: 0.804,
                g: 0.522,
                b: 0.247,
                a: 1.0,
            }),
            "pink" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.753,
                b: 0.796,
                a: 1.0,
            }),
            "plum" => Ok(FillStyle::Color {
                r: 0.867,
                g: 0.627,
                b: 0.867,
                a: 1.0,
            }),
            "powderblue" => Ok(FillStyle::Color {
                r: 0.69,
                g: 0.878,
                b: 0.902,
                a: 1.0,
            }),
            "purple" => Ok(FillStyle::Color {
                r: 0.5,
                g: 0.0,
                b: 0.5,
                a: 1.0,
            }),
            "red" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            "rosybrown" => Ok(FillStyle::Color {
                r: 0.737,
                g: 0.561,
                b: 0.561,
                a: 1.0,
            }),
            "royalblue" => Ok(FillStyle::Color {
                r: 0.255,
                g: 0.412,
                b: 0.882,
                a: 1.0,
            }),
            "saddlebrown" => Ok(FillStyle::Color {
                r: 0.545,
                g: 0.271,
                b: 0.075,
                a: 1.0,
            }),
            "salmon" => Ok(FillStyle::Color {
                r: 0.98,
                g: 0.502,
                b: 0.447,
                a: 1.0,
            }),
            "sandybrown" => Ok(FillStyle::Color {
                r: 0.957,
                g: 0.643,
                b: 0.376,
                a: 1.0,
            }),
            "seagreen" => Ok(FillStyle::Color {
                r: 0.18,
                g: 0.545,
                b: 0.341,
                a: 1.0,
            }),
            "seashell" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.961,
                b: 0.933,
                a: 1.0,
            }),
            "sienna" => Ok(FillStyle::Color {
                r: 0.627,
                g: 0.322,
                b: 0.176,
                a: 1.0,
            }),
            "silver" => Ok(FillStyle::Color {
                r: 0.75,
                g: 0.75,
                b: 0.75,
                a: 1.0,
            }),
            "skyblue" => Ok(FillStyle::Color {
                r: 0.529,
                g: 0.808,
                b: 0.922,
                a: 1.0,
            }),
            "slateblue" => Ok(FillStyle::Color {
                r: 0.416,
                g: 0.353,
                b: 0.804,
                a: 1.0,
            }),
            "slategray" => Ok(FillStyle::Color {
                r: 0.439,
                g: 0.502,
                b: 0.565,
                a: 1.0,
            }),
            "snow" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.98,
                b: 0.98,
                a: 1.0,
            }),
            "springgreen" => Ok(FillStyle::Color {
                r: 0.0,
                g: 1.0,
                b: 0.498,
                a: 1.0,
            }),
            "steelblue" => Ok(FillStyle::Color {
                r: 0.275,
                g: 0.51,
                b: 0.706,
                a: 1.0,
            }),
            "tan" => Ok(FillStyle::Color {
                r: 0.824,
                g: 0.706,
                b: 0.549,
                a: 1.0,
            }),
            "teal" => Ok(FillStyle::Color {
                r: 0.0,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            }),
            "thistle" => Ok(FillStyle::Color {
                r: 0.847,
                g: 0.749,
                b: 0.847,
                a: 1.0,
            }),
            "tomato" => Ok(FillStyle::Color {
                r: 1.0,
                g: 0.388,
                b: 0.278,
                a: 1.0,
            }),
            "turquoise" => Ok(FillStyle::Color {
                r: 0.251,
                g: 0.878,
                b: 0.816,
                a: 1.0,
            }),
            "violet" => Ok(FillStyle::Color {
                r: 0.933,
                g: 0.51,
                b: 0.933,
                a: 1.0,
            }),
            "wheat" => Ok(FillStyle::Color {
                r: 0.961,
                g: 0.871,
                b: 0.702,
                a: 1.0,
            }),
            "white" => Ok(FillStyle::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            "whitesmoke" => Ok(FillStyle::Color {
                r: 0.961,
                g: 0.961,
                b: 0.961,
                a: 1.0,
            }),
            "yellow" => Ok(FillStyle::Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            "yellowgreen" => Ok(FillStyle::Color {
                r: 0.604,
                g: 0.804,
                b: 0.196,
                a: 1.0,
            }),
            _ => Err(format!("Unknown color name: {}", name)),
        }
    }

    /// Get the RGBA color values for rendering
    pub fn get_rgba(&self) -> (f32, f32, f32, f32) {
        match self {
            FillStyle::Color { r, g, b, a } => (*r, *g, *b, *a),
            _ => (0.0, 0.0, 0.0, 1.0), // Default to black for unsupported types
        }
    }

    // Add method to apply global alpha
    pub fn with_global_alpha(&self, global_alpha: f32) -> Self {
        match self {
            FillStyle::Color { r, g, b, a } => FillStyle::Color {
                r: *r,
                g: *g,
                b: *b,
                a: a * global_alpha,
            },
            _ => self.clone(),
        }
    }
}
