use crate::file_manager::export::errors::ExportError;
use crate::world::TideWorld;
use typst::compile;
use typst::diag::Warned;
use typst::layout::PagedDocument;

pub mod errors;
pub mod pdf;
pub mod svg;
pub mod template;

#[derive(Debug, Clone)]
pub enum ExportType {
    PDF,
    SVG,
    Template,
}

/// Compiles the main Typst source in the given [`TideWorld`] into a [`PagedDocument`].
///
/// Captures any warnings during compilation and logs them to `stderr`.
///
/// # Errors
///
/// Returns a [`ExportError::CompilationError`] if the Typst compilation fails.
fn compile_document(world: &TideWorld) -> Result<PagedDocument, ExportError> {
    let Warned { output, warnings } = compile(world);
    if !warnings.is_empty() {
        eprintln!("Typst warnings: {:?}", warnings);
    }
    output.map_err(ExportError::CompilationError)
}
