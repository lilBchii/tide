use crate::file_manager::file::get_relative_path;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::PathBuf;
use typst::foundations::Bytes;
use typst::syntax::{FileId, Source, VirtualPath};

/// Represents a file that was imported, either as an asset or a Typst source file.
pub enum ImportedFile {
    /// Represents a binary file (e.g. image or font) to be used as an asset.
    Asset {
        /// Unique file identifier, using a virtual path relative to the project root.
        file_id: FileId,
        /// Raw binary contents of the asset.
        bytes: Bytes,
    },
    /// Represents a `.typ` Typst source file.
    TypstSource {
        /// Unique file identifier, using a virtual path relative to the project root.
        file_id: FileId,
        /// The actual source code of the Typst file.
        source: Source,
    },
}

/*
pub fn load_files(paths: &Vec<PathBuf>) -> Result<Vec<ImportedFile>, Error>{
    let mut imported_files: Vec<ImportedFile> = Vec::new();
    for path in paths {
        let import_file = import_file(&path)?;
        imported_files.push(import_file);
    }

    Ok(imported_files)
}
 */

/// Loads a single file from disk, detecting whether it is a Typst source or an asset.
///
/// Builds a virtual path for the file relative to the project root.
///
/// # Errors
///
/// Returns an [`Error`] if the file cannot be read, the extension is unknown, or the path is invalid.
pub fn load_file(
    path: &PathBuf,
    root: &PathBuf,
) -> Result<ImportedFile, Error> {
    if let Some(extension) = path.extension().and_then(OsStr::to_str) {
        match extension {
            "typ" => load_source_file(path, root),
            _ => {
                let mut file = File::open(path)?;
                let mut buff = Vec::new();
                file.read_to_end(&mut buff)?;
                let bytes = Bytes::new(buff);
                let id = FileId::new(
                    None,
                    VirtualPath::new(
                        get_relative_path(root, path)
                            .ok_or(Error::from(ErrorKind::NotFound))?,
                    ),
                );
                Ok(ImportedFile::Asset { file_id: id, bytes })
            }
        }
    } else {
        Err(Error::new(ErrorKind::Other, "Import error"))
    }
}

/// Loads a `.typ` Typst source file and wraps it as an [`ImportedFile::TypstSource`] instance.
///
/// # Errors
///
/// Returns an [`Error`] if the file cannot be read or the virtual path could not be computed.
fn load_source_file(
    path: &PathBuf,
    root: &PathBuf,
) -> Result<ImportedFile, Error> {
    let content = fs::read_to_string(path)?;
    let id = FileId::new(
        None,
        VirtualPath::new(
            get_relative_path(root, path).ok_or(Error::from(ErrorKind::NotFound))?,
        ),
    );
    let source = Source::new(id, content);
    Ok(ImportedFile::TypstSource {
        file_id: id,
        source,
    })
}

/// Recursively imports all compatible files from a given directory path.
/// Only files with supported extensions (e.g. `.typ`, `.png`, `.svg`) are loaded and wrapped as [`ImportedFile`] variants.
///
/// # Errors
///
/// Returns an [`Error`] if any directory or file access fails during the traversal or import process.
pub fn load_repo(
    path: &PathBuf,
    root: &PathBuf,
) -> Result<Vec<ImportedFile>, Error> {
    let files = recurse_compatible_files(path)?;
    let mut imported_files = vec![];
    for file in files {
        if let Ok(imported_file) = load_file(&file, root) {
            imported_files.push(imported_file);
        }
    }
    Ok(imported_files)
}

/// Recursively gathers all file paths inside the provided directory.
/// This function does not filter files by extension; all file paths are returned.
///
/// # Errors
///
/// Returns an [`Error`] if a directory or file cannot be accessed.
fn recurse_compatible_files(path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut buf = vec![];
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut sub_dir = recurse_compatible_files(&entry.path())?;
            buf.append(&mut sub_dir);
        }

        if meta.is_file() {
            buf.push(entry.path());
        }
    }

    Ok(buf)
}
