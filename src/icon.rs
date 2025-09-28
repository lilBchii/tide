use iced::widget::{text, Text};
use iced::Font;

// Code heavily inspired by iced_fontello [1]
//
// [1] : https://github.com/hecrj/iced_fontello

pub const ICON_FONT: &[u8] = include_bytes!("../assets/icons/icons.ttf");

pub fn open_dir<'a>() -> Text<'a> {
    icon("\u{e800}")
}

pub fn close_dir<'a>() -> Text<'a> {
    icon("\u{e801}")
}

pub fn eye<'a>() -> Text<'a> {
    icon("\u{e802}")
}

pub fn flag<'a>() -> Text<'a> {
    icon("\u{e803}")
}

pub fn upload<'a>() -> Text<'a> {
    icon("\u{e804}")
}

pub fn trash<'a>() -> Text<'a> {
    icon("\u{e805}")
}

pub fn help<'a>() -> Text<'a> {
    icon("\u{e806}")
}

pub fn book<'a>() -> Text<'a> {
    icon("\u{e807}")
}

pub fn idea<'a>() -> Text<'a> {
    icon("\u{e808}")
}

pub fn folder<'a>() -> Text<'a> {
    icon("\u{e809}")
}

pub fn download<'a>() -> Text<'a> {
    icon("\u{e80a}")
}

pub fn search<'a>() -> Text<'a> {
    icon("\u{e80b}")
}

pub fn text_file<'a>() -> Text<'a> {
    icon("\u{f0f6}")
}

pub fn packages<'a>() -> Text<'a> {
    icon("\u{f1b3}")
}

pub fn pdf_file<'a>() -> Text<'a> {
    icon("\u{f1c1}")
}

pub fn image_file<'a>() -> Text<'a> {
    icon("\u{f1c5}")
}

fn icon<'a>(codepoint: &'a str) -> Text<'a> {
    text(codepoint).font(Font::with_name("fontello"))
}
