use iced::{
    border::rounded,
    widget::{container::Style as ContainerStyle, svg::Style as SvgStyle, text::Style as TxtStyle},
    Theme,
};

/// Returns the default text color style for the main file in the file tree.
pub fn main_style(theme: &Theme) -> TxtStyle {
    let palette = theme.extended_palette();
    TxtStyle {
        color: Some(palette.background.strong.color),
    }
}

/// Returns the color styling for SVG icons.
pub fn svg_icon(theme: &Theme, _status: iced::widget::svg::Status) -> SvgStyle {
    let palette = theme.extended_palette();
    SvgStyle {
        color: Some(palette.primary.weak.color),
    }
}

/// Returns the style for a container representing a selected file in the file tree.
pub fn selected_file(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();
    ContainerStyle {
        background: Some(palette.primary.base.color.into()),
        border: rounded(4),
        ..Default::default()
    }
}
