use iced::{
    border::rounded,
    widget::{button, container, text},
    Theme,
};

/// Returns the default text color style for the main file in the file tree.
pub fn main_style(theme: &Theme) -> text::Style {
    let palette = theme.extended_palette();
    text::Style {
        color: Some(palette.primary.strong.color),
    }
}

pub fn direntry_button(
    theme: &Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();
    let active = button::Style {
        background: Some(palette.background.base.color.into()),
        border: rounded(4),
        ..Default::default()
    };

    match status {
        button::Status::Active => active,
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weak.color.into()),
            ..active
        },
        button::Status::Pressed => button::Style {
            background: Some(palette.background.strong.color.into()),
            ..active
        },
        button::Status::Disabled => button::Style {
            text_color: palette.background.weak.color,
            ..active
        },
    }
}

pub fn direntry_selected_button(
    theme: &Theme,
    _status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();
    button::Style {
        background: Some(palette.secondary.weak.color.into()),
        border: rounded(4),
        ..Default::default()
    }
}

pub fn drop_down_bg(theme: &Theme) -> container::Style {
    let extended_palette = theme.extended_palette();

    container::Style {
        background: Some(extended_palette.background.weak.color.into()),
        ..Default::default()
    }
}
