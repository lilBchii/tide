use super::serialization::{color_serde, color_serde_maybe};
use iced::{theme::Palette, Color, Theme};
use serde::Deserialize;
use std::borrow::Cow;
use std::path::PathBuf;
use std::{collections::HashMap, fs};

/// Default auto-pairing characters used in the editor.
/// These pairs are automatically inserted when typing in the editor.
const DEFAULT_AUTO_PAIRS: [(char, char); 4] =
    [('(', ')'), ('"', '"'), ('[', ']'), ('{', '}')];
/// Embedded fallback font (Roboto Black) used if no font is found at the configured path.
const DEFAULT_FONT: &[u8] = include_bytes!("../../../assets/fonts/Roboto-Black.ttf");
/// Default path to the Roboto Black font on disk.
const DEFAULT_FONT_PATH: &str = "assets/fonts/Roboto-Black.ttf";

/// Root configuration structure loaded from a TOML file.
///
/// Contains grouped settings for general application behavior, color themes, and editor preferences.
#[derive(Default, Deserialize, Debug)]
pub struct Config {
    /// General application settings (font path, scale factor, etc.).
    pub general: GeneralConfig,
    /// UI color palette configuration.
    pub colors: ColorsConfig,
    /// Editor-specific configuration such as font size and auto-pairs.
    pub editor: EditorConfig,
}

/// Configuration for general application behavior.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case", default = "GeneralConfig::default")]
pub struct GeneralConfig {
    /// Path to the font file used in the application.
    pub font_path: String,
    /// Default font size for UI elements.
    pub font_size: u16,
    /// Scale factor for high-DPI displays.
    pub window_scale_factor: f64,
}

/// Configuration specific to the text editor component.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case", default = "EditorConfig::default")]
pub struct EditorConfig {
    /// Font size used in the text editor.
    pub font_size: u16,
    /// Automatically inserted character pairs (e.g. `(` -> `)`).
    #[serde(default = "default_pairs")]
    pub auto_pairs: HashMap<char, char>,
    pub colors: HighlighterTheme,
}

/// Configuration for UI theme colors.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ColorsConfig {
    /// Background color of the UI.
    #[serde(with = "color_serde")]
    pub background: Color,
    /// Default text color.
    #[serde(with = "color_serde")]
    pub text: Color,
    /// Primary accent color used for key UI elements.
    #[serde(with = "color_serde")]
    pub primary: Color,
    /// Color used to indicate success states.
    #[serde(with = "color_serde")]
    pub success: Color,
    /// Color used to indicate error or danger states.
    #[serde(with = "color_serde")]
    pub danger: Color,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HighlighterTheme {
    #[serde(default, with = "color_serde_maybe")]
    pub function: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub number: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub comment: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub string: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub keyword: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub math_delimiter: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub reference: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub label: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub punctuation: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub escape: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub link: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub raw: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub list_marker: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub list_term: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub math_operator: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub operator: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub interpolated: Option<Color>,
    #[serde(default, with = "color_serde_maybe")]
    pub error: Option<Color>,
}

impl Default for HighlighterTheme {
    fn default() -> Self {
        Self {
            function: Some(Color::from_rgb(0.137, 0.612, 0.678)),
            number: Some(Color::from_rgb(200.0 / 255.0, 85.0 / 255.0, 85.0 / 255.0)),
            comment: Some(Color::from_rgb(130.0 / 255.0, 140.0 / 255.0, 145.0 / 255.0)),
            string: Some(Color::from_rgb(50.0 / 255.0, 158.0 / 255.0, 117.0 / 255.0)),
            keyword: Some(Color::from_rgb(200.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0)),
            math_delimiter: Some(Color::from_rgb(0.137, 0.612, 0.678)),
            reference: Some(Color::from_rgb(50.0 / 255.0, 100.0 / 255.0, 120.0 / 255.0)),
            label: Some(Color::from_rgb(50.0 / 255.0, 100.0 / 255.0, 120.0 / 255.0)),
            punctuation: None,
            escape: None,
            link: Some(Color::from_rgb(180.0 / 255.0, 1.0, 1.0)),
            raw: Some(Color::from_rgb(130.0 / 255.0, 140.0 / 255.0, 145.0 / 255.0)),
            list_marker: None,
            list_term: None,
            math_operator: Some(Color::from_rgb(
                50.0 / 255.0,
                158.0 / 255.0,
                117.0 / 255.0,
            )),
            operator: None,
            interpolated: None,
            error: None,
        }
    }
}

impl Config {
    /// Loads a `Config` from a TOML file at the given path.
    ///
    /// If the file is missing or malformed, returns the default configuration.
    pub fn load(file_path: Option<PathBuf>) -> Self {
        match file_path {
            None => Self::default(),
            Some(file_path) => {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        println!("Configuration file loaded successfully");
                        match toml::from_str(&content) {
                            Ok(conf) => {
                                println!("Configuration file read successfully");
                                return conf;
                            }
                            Err(e) => {
                                eprint!("Configuration file deserialization failed: {e}");
                                println!("Default configuration set instead");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error while loading the configuration file: {e}");
                        println!("Default configuration set instead");
                    }
                }
                Self::default() //an error occurred
            }
        }
    }

    /// Loads the font specified in the configuration.
    ///
    /// Falls back to a default embedded font (`Roboto-Black`) if loading fails.
    ///
    /// Returns the font data as a `Cow` (`Borrowed` if default, `Owned` if read from file).
    //configuration file doesn't exist -> self.general.font_path = DEFAULT_FONT_PATH
    //configuration file exists but the font path is wrong -> DEFAULT_PATH is returned
    //configuration file exists and the font path is correct -> the loaded font by fs::read() is returned
    pub fn retrieve_font(&self) -> Cow<'static, [u8]> {
        //unwrapping is dangerous, we use Cow instead
        //anyway, Iced explicitly uses this type in the signature of its font function (impl Into<Cow<'static, [u8]>>)
        fs::read(&self.general.font_path)
            .map(Cow::Owned) //allocate the Vec<u8>
            .unwrap_or_else(|err| {
                eprintln!("Can't load font: {err}");
                Cow::Borrowed(DEFAULT_FONT) //send the reference
            })
    }
}

impl Default for GeneralConfig {
    /// Returns default general configuration settings.
    fn default() -> Self {
        Self {
            font_path: DEFAULT_FONT_PATH.to_string(),
            font_size: 14,
            window_scale_factor: 1.0,
        }
    }
}

impl Default for EditorConfig {
    /// Returns default editor settings, including auto-pairs.
    fn default() -> Self {
        Self {
            font_size: 14,
            auto_pairs: HashMap::from(DEFAULT_AUTO_PAIRS),
            colors: HighlighterTheme::default(),
        }
    }
}

impl Default for ColorsConfig {
    /// Returns the default color palette.
    fn default() -> Self {
        Self {
            background: Color::from_rgb(0.98, 0.98, 0.98),
            text: Color::from_rgb(0.04, 0.04, 0.04),
            primary: Color::from_rgb(0.137, 0.612, 0.678),
            success: Color::from_rgb(0.7, 0.7, 0.7),
            danger: Color::from_rgb(0.86, 0.08, 0.16),
        }
    }
}

impl From<&ColorsConfig> for Theme {
    /// Converts a `ColorsConfig` into a `Theme` instance for Iced.
    fn from(value: &ColorsConfig) -> Self {
        Theme::custom(
            String::from("config_theme"),
            Palette {
                background: value.background,
                text: value.text,
                primary: value.primary,
                success: value.success,
                danger: value.danger,
            },
        )
    }
}

/// Returns the default auto-pairing character map used in the editor.
fn default_pairs() -> HashMap<char, char> {
    DEFAULT_AUTO_PAIRS.into()
}
