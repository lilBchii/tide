use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

//todo: handle SourceDiagnostic instead of String

/// Describes possible failures during compilation, export, or writing.
#[derive(Debug)]
pub enum ExportError {
    /// The Typst compilation process failed.
    ///
    /// Contains a message detailing the compilation error.
    CompilationError(String),
    /// PDF generation failed after successful compilation.
    ///
    /// Contains a message describing the reason for the failure.
    PdfGenerationError(String),
    /// Writing a file to disk failed.
    ///
    /// Wraps a standard I/O error from the filesystem.
    FileWriteError(io::Error),
    /// Reading a file from disk failed.
    ///
    /// Wraps a standard I/O error from the filesystem.
    FileReadError(io::Error),
}

impl Display for ExportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::CompilationError(msg) => write!(f, "Can't compile document: {}", msg),
            ExportError::PdfGenerationError(msg) => write!(f, "Can't generate PDF: {}", msg),
            ExportError::FileWriteError(err) => write!(f, "Can't write file: {}", err),
            ExportError::FileReadError(err) => write!(f, "Can't read file: {}", err),
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ExportError::FileWriteError(err) => Some(err),
            _ => None,
        }
    }
}

impl Clone for ExportError {
    fn clone(&self) -> Self {
        match self {
            ExportError::CompilationError(msg) => ExportError::CompilationError(msg.clone()),
            ExportError::PdfGenerationError(msg) => ExportError::PdfGenerationError(msg.clone()),
            ExportError::FileWriteError(err) => ExportError::FileWriteError(io::Error::new(err.kind(), err.to_string())),
            ExportError::FileReadError(err) => ExportError::FileReadError(io::Error::new(err.kind(), err.to_string())),
        }
    }
}