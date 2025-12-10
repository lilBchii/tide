use std::ops::Range;

use crate::{
    data::config::appearance::HighlighterTheme,
    font::{FONT_BOLD, FONT_ITALIC, FONT_SEMI_BOLD},
};
use iced::advanced::text::highlighter::{self, Format};
use iced::{Color, Font};

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
        println!("highlighter::change_line");
        self.current_line = line;
    }

    fn highlight_line(
        &mut self,
        line: &str,
    ) -> Self::Iterator<'_> {
        println!("highlighter::highlight_line {}", line);
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
            typst::syntax::Tag::Emph => Highlight::with_font(FONT_ITALIC),
            typst::syntax::Tag::Strong | typst::syntax::Tag::Heading => {
                Highlight::with_font(FONT_BOLD)
            }
            typst::syntax::Tag::Keyword => Highlight::with_color(theme.keyword),
            typst::syntax::Tag::MathDelimiter => {
                Highlight::with_color(theme.math_delimiter)
            }
            typst::syntax::Tag::Ref => Highlight::new(theme.reference, FONT_SEMI_BOLD),
            typst::syntax::Tag::Label => Highlight::new(theme.label, FONT_SEMI_BOLD),
            typst::syntax::Tag::Punctuation => Highlight::with_color(theme.punctuation),
            typst::syntax::Tag::Escape => Highlight::with_color(theme.escape),
            typst::syntax::Tag::Link => Highlight::with_color(theme.link),
            typst::syntax::Tag::Raw => Highlight::new(theme.raw, FONT_SEMI_BOLD),
            typst::syntax::Tag::ListMarker => {
                Highlight::new(theme.list_marker, FONT_SEMI_BOLD)
            }
            typst::syntax::Tag::ListTerm => {
                Highlight::new(theme.list_term, FONT_SEMI_BOLD)
            }
            typst::syntax::Tag::MathOperator => {
                Highlight::with_color(theme.math_operator)
            }
            typst::syntax::Tag::Operator => Highlight::with_color(theme.operator),
            typst::syntax::Tag::Interpolated => Highlight::with_color(theme.interpolated),
            typst::syntax::Tag::Error => Highlight::with_color(theme.error),
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
    pub fn new(
        color: Option<Color>,
        font: Font,
    ) -> Self {
        Self {
            color,
            font: Some(font),
        }
    }

    pub fn with_color(color: Option<Color>) -> Self {
        Self { color, font: None }
    }

    pub fn with_font(font: Font) -> Self {
        Self {
            color: None,
            font: Some(font),
        }
    }

    pub fn to_format(&self) -> Format<Font> {
        Format {
            color: self.color,
            font: self.font,
        }
    }
}
