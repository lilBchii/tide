pub mod load;

/// Defines the type of files that can be uploaded through the import interface.
#[derive(Debug, Clone)]
pub enum UploadType {
    /// Allows importing all supported file types (Typst source and assets).
    All,
    /// Restricts import to only Typst source files.
    Template,
}

/// Supported file extensions for full import (includes source and common asset types).
pub const ALL_TYPES: [&str; 5] = ["typ", "png", "jpg", "jpeg", "svg"];
/// Supported file extensions when importing only Typst templates.
pub const TEMPLATE: [&str; 1] = ["typ"];
