use iced::font::Weight::Bold;
use iced::widget::{
    button, center, container, horizontal_space, opaque, row, text, Column, Row,
};
use iced::{Element, Font, Length};
use typst::syntax::FileId;

use crate::data::style::button::{cancel_button, validate_button};
use crate::data::style::pop_up::{confirm, darker_bg, error, title_text, warning};

/// Represents a pop-up dialog that can display a warning, error, or confirmation message to the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopUpElement {
    /// The type of pop-up (see [`PopUpType`]).
    pop_type: PopUpType,
    /// The main title of the pop-up.
    title: String,
    /// The detailed description or context shown in the pop-up.
    details: String,
}

impl PopUpElement {
    /// Creates a new [`PopUpElement`] with the given type, title, and details.
    pub fn new(
        pop_type: PopUpType,
        title: String,
        details: String,
    ) -> Self {
        Self {
            pop_type,
            title,
            details,
        }
    }

    /// Returns the formatted title to be displayed, combining type and title.
    fn title_text(&self) -> String {
        format!("{} - {}", self.pop_type.text(), self.title)
    }

    /// Returns the formatted details string.
    fn details(&self) -> String {
        format!("=> {}", self.details)
    }

    /// Returns the Iced view representing the pop-up element.
    ///
    /// The view includes the title, details, and appropriate buttons depending on the pop-up type.
    pub fn view(&self) -> Element<Message> {
        let col = Column::new()
            .push(
                text(self.title_text())
                    .font(Font {
                        weight: Bold,
                        ..Font::DEFAULT
                    })
                    .style(title_text)
                    .size(24),
            )
            .push(text(self.details()).size(20))
            .spacing(20);
        let row = match self.pop_type {
            PopUpType::Confirm(id) => button_row(Message::DeleteFile(id)),
            _ => Row::new()
                .push(horizontal_space().width(500))
                .push(
                    button(text("Ok"))
                        .on_press(Message::HidePopUp)
                        .style(validate_button),
                )
                .spacing(20),
        };

        let col = col.push(row);

        opaque(
            center(opaque(
                container(col).width(Length::Shrink).padding(10).style(
                    match self.pop_type {
                        PopUpType::Warning => warning,
                        PopUpType::Error => error,
                        PopUpType::Confirm(_) => confirm,
                    },
                ),
            ))
            .style(darker_bg),
        )
    }
}

/// Represents the different types of pop-up dialogs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopUpType {
    /// A warning message.
    Warning,
    /// An error message.
    Error,
    /// A confirmation dialog tied to a file identified by its [`FileId`].
    Confirm(FileId),
}

impl PopUpType {
    /// Returns a string representation of the pop-up type (e.g., "WARNING", "ERROR", "CONFIRM").
    fn text(&self) -> String {
        match self {
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Confirm(_) => "CONFIRM",
        }
        .to_owned()
    }
}

/// Messages used in the context of pop-up dialog interactions.
#[derive(Debug, Clone)]
pub enum Message {
    /// Triggered when a pop-up should be shown.
    ShowPopUp(PopUpElement),
    /// Triggered to dismiss an active pop-up.
    HidePopUp,
    /// Triggered when the user confirms deletion of a file.
    DeleteFile(FileId),
}

/// Returns a [`Row`] widget containing "Cancel" and "Ok" buttons for confirmation pop-ups.
///
/// `on_press` is the message that will be sent when "Ok" is pressed.
fn button_row<'a>(on_press: Message) -> Row<'a, Message> {
    row![
        horizontal_space().width(500),
        button(text("Cancel"))
            .on_press(Message::HidePopUp)
            .style(cancel_button),
        horizontal_space().width(20),
        button(text("Ok")).on_press(on_press).style(validate_button)
    ]
}
