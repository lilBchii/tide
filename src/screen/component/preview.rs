use iced::advanced::svg::Handle;

/// Represents the current state of the document preview, including rendering handles and display mode.
pub struct Preview {
    /// A collection of rendering handles associated with the preview.
    ///
    /// This may be `None` if no preview is currently loaded or available.
    pub handle: Option<Vec<Handle>>,
    /// Indicates whether the preview should be displayed in inverted (e.g., dark mode) colors.
    pub is_inverted: bool,
}

impl Preview {
    /// Creates a new [`Preview`] instance with no handle and non-inverted colors.
    pub fn new() -> Self {
        Self {
            handle: None,
            is_inverted: false,
        }
    }
}
