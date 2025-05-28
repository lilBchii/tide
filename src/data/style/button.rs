use iced::{
    border,
    widget::button::{Status, Style},
    Border, Color, Theme,
};

/// Returns a success-themed style for a "validate" button based on the current theme and widget status.
///
/// The style changes color and border appearance depending on the [`Status`].
pub fn validate_button(
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let active = Style {
        background: Some(palette.success.weak.color.into()),
        text_color: palette.success.base.text,
        border: Border {
            color: palette.success.strong.color,
            width: 1.5,
            radius: 3.0.into(),
        },
        ..Default::default()
    };
    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(palette.success.base.color.into()),
            ..active
        },
        Status::Pressed => Style {
            background: Some(palette.success.strong.color.into()),
            ..active
        },
        Status::Disabled => Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.strong.color,
            ..active
        },
    }
}

/// Returns a danger-themed style for a "cancel" button using the theme's danger palette.
///
/// Adjusts colors dynamically based on the button's [`Status`].
pub fn cancel_button(
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let active = Style {
        background: Some(palette.danger.base.color.into()),
        text_color: palette.danger.base.text,
        border: border::rounded(3),
        ..Default::default()
    };
    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(palette.danger.strong.color.into()),
            text_color: palette.danger.strong.text,
            ..active
        },
        Status::Pressed => Style {
            background: Some(palette.danger.weak.color.into()),
            ..active
        },
        Status::Disabled => Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.strong.color,
            ..active
        },
    }
}

/// Returns a primary-colored style used for toolbar buttons.
///
/// Uses colors from the theme's primary palette and visually reacts to hover, press, and disabled states.
pub fn toolbar_button(
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let active = Style {
        background: Some(palette.primary.weak.color.into()),
        text_color: palette.primary.base.text,
        border: Border {
            color: palette.primary.base.color,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };
    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(palette.primary.base.color.into()),
            ..active
        },
        Status::Pressed => Style {
            background: Some(palette.primary.strong.color.into()),
            ..active
        },
        Status::Disabled => Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.strong.color,
            ..active
        },
    }
}

/// Returns a minimalistic style for dropdown menu buttons with contextual highlighting on interaction.
///
/// The style is typically borderless until hovered or pressed, at which point it gains color and border.
pub fn drop_down_menu_button(
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let active = Style {
        background: None,
        text_color: palette.primary.base.text,
        ..Default::default()
    };
    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(palette.primary.base.color.into()),
            border: Border {
                color: palette.primary.base.color,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..active
        },
        Status::Pressed => Style {
            background: Some(palette.success.strong.color.into()),
            border: Border {
                color: palette.success.base.color,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..active
        },
        Status::Disabled => Style {
            text_color: palette.background.strong.color,
            ..active
        },
    }
}

/// Returns a clean, borderless style for buttons used in file-related UI elements.
///
/// On press, the button adopts a primary highlight color.
pub fn files_button(
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let active = Style {
        background: None,
        text_color: palette.primary.base.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };
    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(palette.primary.weak.color.into()),
            ..active
        },
        Status::Pressed => Style {
            background: Some(palette.primary.strong.color.into()),
            ..active
        },
        Status::Disabled => Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.strong.color,
            ..active
        },
    }
}
