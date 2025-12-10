use iced::{
    widget::{row, space, text, text_editor::Position},
    Alignment, Element, Length,
};

use crate::screen::editing::Message;

const SPACING: f32 = 20.0;

/// Builds and returns the view for the status bar at the bottom of the editor.
///
/// The status bar displays:
/// - the current cursor position as line and column numbers ;
/// - the name of the currently opened file ;
/// - a flag indicating whether the file has been saved.
pub fn status_bar_view<'a>(
    cursor_pos: Position,
    current_file: String,
    saved: bool,
) -> Element<'a, Message> {
    row![
        space().width(SPACING),
        text(format! {"{}:{}", cursor_pos.line, cursor_pos.column}),
        text(current_file),
        space().width(Length::Fill),
        text(format! {"saved: {}", saved}),
        space().width(SPACING)
    ]
    .spacing(SPACING)
    .align_y(Alignment::Center)
    .into()
}
