use iced::Font;
use iced_core::font;

pub const FONT_BOLD: Font = Font {
    family: font::Family::Monospace,
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const FONT_ITALIC: Font = Font {
    family: font::Family::Monospace,
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Italic,
};

pub const FONT_SEMI_BOLD: Font = Font {
    family: font::Family::Monospace,
    weight: iced::font::Weight::Semibold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};
