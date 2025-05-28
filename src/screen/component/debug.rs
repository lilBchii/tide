use crate::data::style::button::validate_button;
use crate::data::style::debug::debug_container_style;
use iced::widget::text::Shaping;
use iced::widget::{
    button, column, container, opaque, row, text, Scrollable, Space, Text,
};
use iced::{Element, Fill};

/// A UI component that displays Typst-related error messages in a scrollable debug panel.
#[derive(Debug, Clone)]
pub struct DebugZone {
    /// A string containing the formatted Typst errors.
    typst_errors: String,
}

impl DebugZone {
    /// Creates a new [`DebugZone`] instance with the given error message.
    pub fn new(typst_errors: String) -> Self {
        Self { typst_errors }
    }

    /// Returns the string of formatted errors.
    fn errors(&self) -> String {
        format!("{}", self.typst_errors)
    }

    /// Returns the Iced view for this debug zone.
    ///
    /// The view includes a close button and a scrollable area for the error text.
    pub fn view(&self) -> Element<Message> {
        opaque(
            container(column![
                row![
                    Space::with_width(Fill),
                    button(Text::new("X").shaping(Shaping::Advanced))
                        .on_press(Message::HideErrors)
                        .style(validate_button),
                ],
                Scrollable::new(text(self.errors()).shaping(Shaping::Advanced).size(18))
            ])
            .style(debug_container_style)
            .width(Fill)
            .height(250),
        )
    }
}

/// Messages handled by the debug zone component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Triggered when errors should be shown in a debug zone.
    ShowErrors(DebugZone),
    /// Triggered when the error display should be hidden
    /// (that means, when the debug zone should be hidden).
    HideErrors,
}
