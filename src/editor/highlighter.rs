use std::ops::Range;

use iced::{Color, Font, Theme as IcedTheme};
use iced_core::text::highlighter::{self, Format};

use crate::data::config::appearance::HighlighterTheme;

pub struct Highlighter {
    settings: Settings,
    current_line: usize,
}

impl highlighter::Highlighter for Highlighter {
    type Settings = Settings;
    type Highlight = Highlight;
    type Iterator<'a> = Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            settings: settings.clone(),
            current_line: 0,
        }
    }

    fn update(
        &mut self,
        new_settings: &Self::Settings,
    ) {
        self.settings = new_settings.clone();
        self.current_line = 0;
    }

    fn change_line(
        &mut self,
        line: usize,
    ) {
        self.current_line = line;
    }

    fn highlight_line(
        &mut self,
        line: &str,
    ) -> Self::Iterator<'_> {
        self.current_line += 1;
        let mut vec = vec![];
        let root = typst::syntax::parse(line);
        highlight_tree(
            &mut vec,
            &typst::syntax::LinkedNode::new(&root),
            &self.settings.theme,
        );

        Box::new(vec.into_iter())
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn highlight_tree(
    tags: &mut Vec<(Range<usize>, Highlight)>,
    node: &typst::syntax::LinkedNode,
    theme: &HighlighterTheme,
) {
    if let Some(tag) = typst::syntax::highlight(node) {
        let highlight = match tag {
            typst::syntax::Tag::Comment => Highlight::with_color(theme.comment),
            typst::syntax::Tag::Function => Highlight::with_color(theme.function),
            typst::syntax::Tag::String => Highlight::with_color(theme.string),
            typst::syntax::Tag::Number => Highlight::with_color(theme.number),
            typst::syntax::Tag::Emph => Highlight::with_font(Font {
                family: iced::font::Family::Monospace,
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Italic,
            }),
            typst::syntax::Tag::Strong | typst::syntax::Tag::Heading => {
                Highlight::with_font(Font {
                    family: iced::font::Family::Monospace,
                    weight: iced::font::Weight::Bold,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
            }
            typst::syntax::Tag::Keyword => Highlight::with_color(theme.keyword),
            _ => Highlight::none(),
        };
        tags.push((node.range(), highlight));
    }

    for child in node.children() {
        highlight_tree(tags, &child, theme);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub theme: HighlighterTheme,
    pub extension: String,
}

pub struct Highlight {
    color: Option<Color>,
    font: Option<Font>,
}

impl Highlight {
    pub fn with_color(color: Color) -> Self {
        Self {
            color: Some(color),
            font: None,
        }
    }

    pub fn with_font(font: Font) -> Self {
        Self {
            color: None,
            font: Some(font),
        }
    }

    pub fn none() -> Self {
        Self {
            color: None,
            font: None,
        }
    }

    pub fn to_format(&self) -> Format<Font> {
        Format {
            color: self.color,
            font: self.font,
        }
    }
}
