use iced::{
    border,
    widget::{container::Style as ContainerStyle, text::Style as TxtStyle},
    Color, Theme,
};

/// Returns the styling for title text in pop-up elements.
pub fn title_text(theme: &Theme) -> TxtStyle {
    let palette = theme.extended_palette();
    TxtStyle {
        color: Some(palette.background.strong.color),
    }
}

/// Returns the visual style for containers meant to display warning messages.
pub fn warning(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();
    ContainerStyle {
        background: Some(palette.danger.weak.color.into()),
        border: border::rounded(4),
        ..Default::default()
    }
}

/// Returns the style for containers indicating critical errors.
pub fn error(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();
    ContainerStyle {
        background: Some(palette.danger.strong.color.into()),
        border: border::rounded(4),
        ..Default::default()
    }
}

/// Returns the style for confirmation dialogs or neutral action containers for pop-up elements.
pub fn confirm(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();
    ContainerStyle {
        background: Some(palette.background.base.color.into()),
        border: border::rounded(4)
            .color(palette.background.weak.color)
            .width(2.0),
        ..Default::default()
    }
}

/// Returns a semi-transparent black background style, used for modals or overlays.
pub fn darker_bg(_theme: &Theme) -> ContainerStyle {
    ContainerStyle {
        background: Some(
            Color {
                a: 0.5,
                ..Color::BLACK
            }
            .into(),
        ),
        ..Default::default()
    }
}
