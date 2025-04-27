use iced::{border, widget::container::Style, Theme};

/// Returns the tooltip container style, using the theme's primary color.
pub fn tooltip_box(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(palette.primary.weak.color.into()),
        text_color: Some(palette.primary.base.text),
        border: border::rounded(4),
        ..Default::default()
    }
}
