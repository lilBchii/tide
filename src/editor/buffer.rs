use std::{fs, path::PathBuf};

use iced::widget::text_editor::Content;

/// Represents an editable buffer in the editor.
#[derive(Debug)]
pub struct Buffer {
    /// The content of the buffer.
    pub content: Content,
    /// Indicates whether the buffer content is saved to disk.
    pub is_saved: bool,
}

impl Buffer {
    /// Creates a new, empty buffer.
    pub fn new() -> Self {
        Self {
            content: Content::new(),
            is_saved: false,
        }
    }

    /// Creates a buffer from existing content.
    pub fn from_content(content: Content) -> Self {
        Self {
            content,
            is_saved: false,
        }
    }

    /// Loads a buffer from a file path.
    /// Returns an [`std::io::Error`] if the file cannot be read.
    pub fn from_path(path: &PathBuf) -> Result<Self, std::io::Error> {
        let str = fs::read_to_string(path)?;
        Ok(Self {
            content: Content::with_text(&str),
            is_saved: true,
        })
    }

    /// Replaces the current content of the buffer with the specified `content`.
    pub fn fill(&mut self, content: Content) {
        self.content = content;
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            content: Content::with_text(self.content.text().as_str()),
            is_saved: self.is_saved,
        }
    }
}
