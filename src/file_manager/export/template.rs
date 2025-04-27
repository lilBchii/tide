use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use crate::file_manager::export::errors::ExportError;

/// Exports a `.typ` template file by copying it to the specified output directory.
/// This function reads the content of the given source file and writes it into the target
/// directory, preserving the original file name.
///
/// Returns the full path to the newly created file in the output directory upon success.
///
/// # Errors
///
/// Returns an [`ExportError::FileReadError`] if reading the source file fails, or
/// [`ExportError::FileWriteError`] if writing to the destination fails.
pub async fn export_template(file_path: PathBuf, output_path: PathBuf) -> Result<PathBuf, ExportError> {
    let file_content = fs::read(&file_path).map_err(|e| ExportError::FileReadError(e))?;
    let output_path = output_path.join(
        file_path
            .file_name()
            .ok_or(
                ExportError::FileReadError(Error::new(ErrorKind::Other, "Can't read file name"))
            )?
    );
    fs::write(&output_path, file_content).map_err(|e| ExportError::FileWriteError(e))?;

    Ok(output_path)
}