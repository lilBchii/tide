use crate::file_manager::export::ExportType;
use crate::screen::{component::toolbar, editing};
use iced::{
    keyboard::{key, Key},
    widget::text_editor::{Binding, KeyPress, Motion},
};

/// Maps key presses to editor bindings.
/// Returns an optional [`Binding`] corresponding to the key combination to a [`editing::Message`].
///
/// This includes:
/// - `Tab` --> Add four spaces
/// - `Ctrl + S` --> Force preview and save the current file
/// - `Ctrl + Arrow Right` --> Move to the right boundary of a word
/// - `Ctrl + Shift + Arrow Right` --> Move to the right boundary of a word and select all of its characters
/// - `Ctrl + Arrow Left` --> Move to the left boundary of a word
/// - `Ctrl + Shift + Arrow Left` --> Move to the left boundary of a word and select all of its characters
/// - `Del` --> Delete the next character
/// - `Ctrl + O` --> Move to the end of the line and break the current line
/// - `Ctrl + E` --> Export current project as a PDF
/// - `Ctrl + Space` --> Open the autocomplete context
pub fn bindings(key_press: KeyPress) -> Option<Binding<editing::Message>> {
    match key_press.key.as_ref() {
        Key::Named(key::Named::Tab) => Some(Binding::Sequence(vec![Binding::Insert(' '); 4])),
        Key::Character("s") if key_press.modifiers.command() => Some(Binding::Sequence(vec![
            Binding::Custom(editing::Message::ToolBar(toolbar::Message::ForcePreview)),
            Binding::Custom(editing::Message::ToolBar(toolbar::Message::SaveFile(false))),
        ])),

        Key::Named(key::Named::ArrowRight)
            if key_press.modifiers.command() && key_press.modifiers.shift() =>
        {
            Some(Binding::Select(Motion::WordRight))
        }
        Key::Named(key::Named::ArrowLeft)
            if key_press.modifiers.command() && key_press.modifiers.shift() =>
        {
            Some(Binding::Select(Motion::WordLeft))
        }
        Key::Named(key::Named::ArrowRight) if key_press.modifiers.command() => {
            Some(Binding::Move(Motion::WordRight))
        }
        Key::Named(key::Named::ArrowLeft) if key_press.modifiers.command() => {
            Some(Binding::Move(Motion::WordLeft))
        }
        Key::Named(key::Named::Delete) => Some(Binding::Delete),
        Key::Character("o") if key_press.modifiers.command() => Some(Binding::Sequence(vec![
            Binding::Move(Motion::End),
            Binding::Enter,
        ])),
        Key::Character("e") if key_press.modifiers.command() => Some(Binding::Custom(
            editing::Message::ToolBar(toolbar::Message::Export(ExportType::PDF)),
        )),
        Key::Named(key::Named::Space) if key_press.modifiers.command() => {
            Some(Binding::Custom(editing::Message::Autocomplete))
        }
        _ => Binding::from_key_press(key_press),
    }
}
