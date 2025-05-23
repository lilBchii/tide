use iced::widget::container;
use iced::{Border, Theme};

/// Returns the style for debug containers.
///
/// Includes a subtle background, a colored border, and a contrasting text color
/// to easily distinguish debug areas from the rest of the UI.
pub fn debug_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: palette.primary.weak.color,
            radius: 3.into(),
            width: 1.0,
        },
        text_color: Some(palette.danger.base.color.into()),
        ..Default::default()
    }
}
