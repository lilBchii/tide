use std::path::PathBuf;
use typst::layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use crate::file_manager::export::compile_document;
use crate::file_manager::export::errors::ExportError;
use crate::world::TideWorld;

/// Exports the compiled document as a PDF at the specified `output_path`.
///
/// Appends the `.pdf` extension automatically. Uses the provided [`PdfOptions`].
///
/// # Errors
///
/// Returns a [`ExportError`] if compilation, PDF generation, or file writing fails.
pub async fn export_pdf(
    world: TideWorld,
    output_path: PathBuf,
    pdf_options: PdfOptions<'_>,
) -> Result<PathBuf, ExportError> {
    let document = compile_document(&world)?;
    let pdf_content = generate_pdf(&document, &pdf_options)?;
    let output_path= output_path.with_extension("pdf");
    write_pdf(output_path.to_str().unwrap(), &pdf_content)?;
    Ok(PathBuf::from(output_path))
}

/// Converts a compiled Typst [`PagedDocument`] into a PDF byte vector.
///
/// # Errors
///
/// Returns a [`ExportError::PdfGenerationError`] on failure.
fn generate_pdf(
    document: &PagedDocument,
    pdf_options: &PdfOptions,
) -> Result<Vec<u8>, ExportError> {
    pdf(document, pdf_options).map_err(|e| ExportError::PdfGenerationError(format!("{:?}", e)))
}

/// Writes the PDF byte content to the specified file path.
///
/// # Errors
///
/// Returns a [`ExportError::FileWriteError`] if writing fails.
fn write_pdf(output_path: &str, content: &[u8]) -> Result<(), ExportError> {
    std::fs::write(output_path, content).map_err(|e| ExportError::FileWriteError(e))
}