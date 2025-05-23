//Credits to the source inspiration: https://github.com/squidowl/halloy/blob/main/data/src/appearance/theme.rs
use iced::Color;

/// Parses a hexadecimal color string (e.g., `#RRGGBB` or `#RRGGBBAA`) into a [`Color`].
/// `hex` is a string slice containing a 6-digit or 8-digit hexadecimal color with a leading `#`.
///
/// This returns an `Option<Color>` representing the parsed color, or `None` if the format is invalid.
pub fn hex_to_color(hex: &str) -> Option<Color> {
    if hex.len() == 7 || hex.len() == 9 {
        let hash = &hex[0..1];
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);
        let a = (hex.len() == 9)
            .then(|| u8::from_str_radix(&hex[7..9], 16).ok())
            .flatten();

        return match (hash, r, g, b, a) {
            ("#", Ok(r), Ok(g), Ok(b), None) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }),
            ("#", Ok(r), Ok(g), Ok(b), Some(a)) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }),
            _ => None,
        };
    }

    None
}

/// Converts a [`Color`] into a hexadecimal string in the form `#RRGGBB` (or `#RRGGBBAA` if alpha is not 255).
pub fn color_to_hex(color: Color) -> String {
    use std::fmt::Write;

    let mut hex = String::with_capacity(9);

    let [r, g, b, a] = color.into_rgba8();

    let _ = write!(&mut hex, "#");
    let _ = write!(&mut hex, "{:02X}", r);
    let _ = write!(&mut hex, "{:02X}", g);
    let _ = write!(&mut hex, "{:02X}", b);

    if a < u8::MAX {
        let _ = write!(&mut hex, "{:02X}", a);
    }

    hex
}

/// `SerDe` module for serializing and deserializing [`Color`] values to and from hexadecimal strings.
///
/// This is used when reading or writing themes in configuration files.
pub(crate) mod color_serde {
    use iced::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserializes a hexadecimal string (e.g., `#FFAA00`) into a [`Color`].
    ///
    /// Invalid or unrecognized formats default to `Color::TRANSPARENT`.
    ///
    /// # Errors
    ///
    /// Returns a `SerDe` deserialization error if the string is not valid UTF-8 or fails to parse.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)
            .map(|hex| super::hex_to_color(&hex))?
            .unwrap_or(Color::TRANSPARENT))
    }

    /// Serializes a [`Color`] as a hexadecimal string (e.g., `#RRGGBB` or `#RRGGBBAA`).
    ///
    /// # Errors
    ///
    /// Returns a `SerDe` serialization error if writing fails.
    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::color_to_hex(*color).serialize(serializer)
    }
}

/// `SerDe` module for serializing and deserializing optional [`Color`] values (`Option<Color>`)
/// to and from hexadecimal strings.
///
/// Used for configuration fields where colors are optional.
pub(crate) mod color_serde_maybe {
    use iced::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserializes an optional hexadecimal string into an `Option<Color>`.
    ///
    /// Returns `None` if the field is missing or the hex is invalid.
    ///
    /// # Errors
    ///
    /// Returns a `SerDe` deserialization error if the input is not a valid string.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Option::<String>::deserialize(deserializer)?.and_then(|hex| super::hex_to_color(&hex)))
    }

    /// Serializes an `Option<Color>` as a hexadecimal string if `Some`, or as `null` if `None`.
    ///
    /// # Errors
    ///
    /// Returns a `SerDe` serialization error if writing fails.
    pub fn serialize<S>(color: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        color.map(super::color_to_hex).serialize(serializer)
    }
}
