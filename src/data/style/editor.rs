use iced::{
    widget::text_editor::{Status, Style},
    Border, Theme,
};

/// Returns the style for the text editor widget, including background, border, selection, and text colors.
///
/// Reacts to the editor's [`Status`] with color and border adjustments.
pub fn editor_style(
    theme: &Theme,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let active = Style {
        background: palette.background.base.color.into(),
        border: Border {
            color: palette.primary.weak.color,
            radius: 3.into(),
            width: 1.0,
        },
        selection: palette.success.weak.color,
        icon: palette.background.weak.text,
        placeholder: palette.background.strong.color,
        value: palette.background.base.text,
    };
    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: palette.primary.base.color,
                ..active.border
            },
            ..active
        },
        Status::Focused { is_hovered: _ } => Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            border: Border {
                color: palette.danger.base.color,
                ..active.border
            },
            ..active
        },
    }
}
