use iced::font::Family;
use iced::Font;

pub const APP_FONT_FAMILY_NAME: &str = "Gully";

pub const APP_REG_BYTES: &[u8] =
    include_bytes!("../assets/fonts/gully/Gully-Regular.ttf");
pub const APP_SEMI_BOLD_BYTES: &[u8] =
    include_bytes!("../assets/fonts/gully/Gully-SemiBold.ttf");
pub const APP_BOLD_BYTES: &[u8] = include_bytes!("../assets/fonts/gully/Gully-Bold.ttf");
pub const APP_ITALIC_BYTES: &[u8] =
    include_bytes!("../assets/fonts/gully/Gully-Light.ttf");

pub const EDITOR_FONT_FAMILY_NAME: &str = "MD IO";

pub const EDITOR_REG_BYTES: &[u8] =
    include_bytes!("../assets/fonts/md_io_typeface/MDIO-Regular.ttf");
pub const EDITOR_SEMI_BOLD_BYTES: &[u8] =
    include_bytes!("../assets/fonts/md_io_typeface/MDIO-Semibold.ttf");
pub const EDITOR_BOLD_BYTES: &[u8] =
    include_bytes!("../assets/fonts/md_io_typeface/MDIO-Bold.ttf");
pub const EDITOR_ITALIC_BYTES: &[u8] =
    include_bytes!("../assets/fonts/md_io_typeface/MDIO-Italic.ttf");

pub const FONT_BOLD: Font = Font {
    family: Family::Name(EDITOR_FONT_FAMILY_NAME),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const FONT_ITALIC: Font = Font {
    family: Family::Name(EDITOR_FONT_FAMILY_NAME),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Italic,
};

pub const FONT_SEMI_BOLD: Font = Font {
    family: Family::Name(EDITOR_FONT_FAMILY_NAME),
    weight: iced::font::Weight::Semibold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};
