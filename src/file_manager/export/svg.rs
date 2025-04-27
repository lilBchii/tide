use std::ffi::OsStr;
use std::path::PathBuf;
use typst::layout::PagedDocument;
use typst_svg::svg;
use crate::file_manager::export::compile_document;
use crate::file_manager::export::errors::ExportError;
use crate::world::TideWorld;

/// Exports each page of the compiled Typst document as an individual SVG file.
///
/// SVG files are named using the base of `output_path` with page indices appended.
///
/// # Errors
///
/// Returns a [`ExportError`] if compilation or writing fails.
pub async fn export_svg(world: TideWorld, output_path: PathBuf) -> Result<PathBuf, ExportError> {
    let document = compile_document(&world)?;
    let svg_content = generate_svg(&document);
    let output_base_name = output_path.file_name().unwrap_or(OsStr::new("output"));
    write_svg(output_base_name.to_str().unwrap(), svg_content)?;
    Ok(PathBuf::from(output_base_name))
}

//EXPORT EACH PAGE/FRAME
/// Converts each page of a [PagedDocument] into SVG string representations.
fn generate_svg(document: &PagedDocument) -> Vec<String> {
    document.pages.iter().map(|page| svg(page)).collect()
}

/// Writes each SVG string to disk as a separate file using the given base name.
///
/// Each file is named `<base>-<page_index>.svg`.
///
/// # Errors
///
/// Returns a [`ExportError::FileWriteError`] on write failure.
fn write_svg(output_base_name: &str, content: Vec<String>) -> Result<(), ExportError> {
    for (i, page) in content.iter().enumerate() {
        let output_path = output_base_name.to_owned() + "-" + i.to_string().as_str() + ".svg";
        std::fs::write(output_path, page).map_err(|e| ExportError::FileWriteError(e))?
    }
    Ok(())
}

/// Compiles and previews all document pages as in-memory SVG strings. This is used for project preview
/// within the application.
///
/// This does not write any files.
///
/// # Errors
///
/// Returns a [`ExportError`] if the document fails to compile.
pub async fn preview_svg(world: TideWorld) -> Result<Vec<String>, ExportError> {
    let document = compile_document(&world)?;
    Ok(generate_svg(&document))
}

/*
ASYNC SVG GENERATION
... is not possible currently in Iced
see: https://discord.com/channels/628993209984614400/1355550014990057533/1358593193599439039

A possible workaround is using `resvg` to render to an image concurrently in a Task.
But Typst can also render documents as Pixmap (RGBA pixel buffers).
That means we could generate some Pixmap and load them asynchronously with (iced)::Handle::from_rgba() in a Task.
Then, rendering a lower quality preview when the user resizes it.
With some other optimizations (rendering only the pages that are visible on the screen, or reusing the handles),
we could remove all the freezes and crashes that occurred with SVG.
We would need `tiny-skia` and `typst-render`.

fn test_generate_render(document: &PagedDocument) -> Vec<Pixmap> {
    document.pages.iter().map(|page| render(page, 3.0)).collect()
}

pub async fn test_preview_raw(world: TideWorld) -> Result<Vec<Pixmap>, ExportError> {
    let document = compile_document(&world)?;
    Ok(test_generate_render(&document))
}

 */