use iced::{
    border,
    widget::{container::Style as ContainerStyle, text::Style as TextStyle, Theme},
};

/// Returns the style used for modal windows.
///
/// Typically applied to elements requiring focus, such as confirmation dialogs or file prompts.
pub fn modal_style(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();
    ContainerStyle {
        background: Some(palette.primary.weak.color.into()),
        border: border::rounded(4),
        ..Default::default()
    }
}

/// Returns the text style used inside modal dialogs.
///
/// Uses the danger base color to draw attention to critical actions.
pub fn modal_text_style(theme: &Theme) -> TextStyle {
    let palette = theme.extended_palette();
    TextStyle {
        color: palette.danger.base.color.into(),
    }
}
