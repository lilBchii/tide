//this file should be moved to a dedicated module

use crate::file_manager::export::errors::ExportError;
use crate::file_manager::file::{get_fonts_path, get_relative_path};
use crate::file_manager::import::load::ImportedFile;
use chrono::{Datelike, FixedOffset, Local, Utc};
use iced::widget::text_editor::Content;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::PathBuf;
use typst::diag::{FileError, FileResult, Warned};
use typst::foundations::{Bytes, Datetime};
use typst::layout::{Abs, PagedDocument};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook, TextElem, TextSize};
use typst::utils::LazyHash;
use typst::{compile, Library, World};
use typst_ide::IdeWorld;

const LIBERTINUS_SERIF_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/LibertinusSerif-Regular.ttf");
const LIBERTINUS_SERIF_BOLD: &[u8] = include_bytes!("../assets/fonts/LibertinusSerif-Bold.ttf");
const LIBERTINUS_SERIF_ITALIC: &[u8] = include_bytes!("../assets/fonts/LibertinusSerif-Italic.ttf");
const LIBERTINUS_SERIF_BOLD_ITALIC: &[u8] =
    include_bytes!("../assets/fonts/LibertinusSerif-BoldItalic.ttf");
const LIBERTINUS_SERIF_SEMIBOLD: &[u8] =
    include_bytes!("../assets/fonts/LibertinusSerif-Semibold.ttf");
const LIBERTINUS_SERIF_MATH: &[u8] = include_bytes!("../assets/fonts/LibertinusMath-Regular.ttf");
const LIBERTINUS_SERIF_INITIALS: &[u8] =
    include_bytes!("../assets/fonts/LibertinusSerifInitials-Regular.ttf");
const NEW_CMM_MATH_REGULAR: &[u8] = include_bytes!("../assets/fonts/NewCMMath-Regular.otf");

/// The typesetting environment for the Tide IDE, implementing the Typst [`World`] trait.
///
/// Provides cached access to files, fonts, and Typst sources required for typesetting.
pub struct TideWorld {
    /// Identifier of the main source file currently being typeset.
    main: FileId,
    /// Lazily-initialized standard Typst library.
    library: LazyHash<Library>,
    /// Font metadata registry generated from loaded fonts.
    book: LazyHash<FontBook>,
    /// Loaded font instances used for rendering.
    fonts: Vec<Font>,
    /// File storage for both assets and Typst sources.
    files: Files,
}

/// Loads all font files from the given directory and parses their faces.
///
/// Returns a list of valid [`Font`] instances found in the folder.
fn load_fonts(font_path: PathBuf) -> Vec<Font> {
    let mut fonts = Vec::new();

    if let Ok(paths) = fs::read_dir(font_path) {
        for path in paths {
            let font = &path.expect("Can't read this path").path();
            println!("font found: {}", font.display());
            if let Ok(font_data) = fs::read(font) {
                let font_bytes = Bytes::new(font_data);
                let loaded_fonts: Vec<_> = Font::iter(font_bytes).collect(); //if the font data contain several faces
                fonts.extend(loaded_fonts);
            }
        }
    }

    fonts
}

/// Retrieves the default "Libertinus" font and returns it as a collection of OpenType fonts.
fn default_fonts() -> Vec<Font> {
    let mut fonts = Vec::new();
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_REGULAR)));
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_BOLD)));
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_ITALIC)));
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_BOLD_ITALIC)));
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_SEMIBOLD)));
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_MATH)));
    fonts.extend(Font::iter(Bytes::new(LIBERTINUS_SERIF_INITIALS)));
    fonts.extend(Font::iter(Bytes::new(NEW_CMM_MATH_REGULAR)));

    fonts
}

impl TideWorld {
    /// Constructs a new [`TideWorld`] instance with an optional asset map.
    ///
    /// Initializes the standard library, loads fonts from disk,
    /// and sets up the font book and file storage.
    pub fn new(main: FileId, assets: Option<HashMap<FileId, Bytes>>) -> Self {
        let library = LazyHash::new(library());

        let mut fonts: Vec<Font> = default_fonts();

        if let Some(user_fonts_path) = get_fonts_path() {
            let user_fonts = load_fonts(user_fonts_path);
            fonts.extend(user_fonts);
        }

        let book = LazyHash::new(FontBook::from_fonts(&fonts));

        TideWorld {
            main,
            library,
            book,
            fonts,
            files: Files {
                assets: assets.unwrap_or_default(),
                sources: Default::default(),
            },
        }
    }

    /// Creates a [`FileId`] from a file system path and project root.
    ///
    /// Returns `None` if the relative path cannot be determined.
    pub fn id_from_path(path: &PathBuf, root: &PathBuf) -> Option<FileId> {
        let relative_path = get_relative_path(root, path)?;
        Some(FileId::new(None, VirtualPath::new(relative_path)))
    }

    /// Replaces the source content for an existing file in the world.
    pub fn reload_source_from_content(&mut self, id: FileId, content: &Content) {
        if let Some(source) = self.files.sources.get_mut(&id) {
            source.replace(content.text().as_str());
        }
    }

    /// Inserts a new Typst source file into the world.
    pub fn add_source(&mut self, file_id: FileId, source: Source) {
        self.files.sources.insert(file_id, source);
    }

    /// Inserts an asset file (e.g. image, binary) into the world.
    fn add_asset(&mut self, file_id: FileId, asset: Bytes) {
        self.files.assets.insert(file_id, asset);
    }

    /// Adds an imported file to the world (either source or asset).
    pub fn add_file(&mut self, file: ImportedFile) {
        match file {
            ImportedFile::Asset { file_id, bytes } => {
                self.add_asset(file_id, bytes);
            }
            ImportedFile::TypstSource { file_id, source } => {
                self.add_source(file_id, source);
            }
        }
    }

    /// Updates the `main` file ID to a new one.
    pub fn change_main(&mut self, id: FileId) {
        self.main = id;
    }

    /// Removes both source and asset entries for a given file ID.
    pub fn remove_file(&mut self, id: FileId) {
        self.files.assets.remove(&id);
        self.files.sources.remove(&id);
    }

    /// Compiles the main Typst source in the given [`TideWorld`] into a [`PagedDocument`].
    ///
    /// Captures any warnings during compilation and logs them to `stderr`.
    ///
    /// # Errors
    ///
    /// Returns a [`ExportError::CompilationError`] if the Typst compilation fails.
    pub async fn compile_document(self) -> Result<PagedDocument, ExportError> {
        println!("compiling...");
        let Warned { output, warnings } = compile(&self);
        if !warnings.is_empty() {
            eprintln!("Typst warnings: {:?}", warnings);
        }
        println!("OK");
        output.map_err(|e| {
            ExportError::CompilationError(format!(
                "{:?}",
                e.iter().map(|s| s.message.clone()).collect::<Vec<_>>()
            ))
        })
    }
}

impl World for TideWorld {
    /// Returns a reference to the standard Typst library. See [`World::library`].
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Returns a reference to the font book used for typesetting. See [`World::book`].
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Returns the file ID of the main document. See [`World::main`].
    fn main(&self) -> FileId {
        self.main
    }

    /// Retrieves a source file by ID, if available.
    ///
    /// Returns an error if the file is not found.
    ///
    /// See [`World::source`].
    fn source(&self, id: FileId) -> FileResult<Source> {
        if let Some(file) = self.files.sources.get(&id) {
            Ok(file.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    /// Retrieves a binary asset file by ID, if available.
    ///
    /// Returns an error if the file is not found.
    ///
    /// See [`World::file`].
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        println!(
            "id: {:?}, found: {:?}",
            id,
            self.files.assets.contains_key(&id)
        );
        if let Some(file) = self.files.assets.get(&id).cloned() {
            Ok(file)
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    /// Retrieves a font by index.
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// See [`World::font`].
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    /// Returns the current date/time. See [`World::today`].
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = Utc::now();
        let with_offset = match offset {
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                now.with_timezone(&FixedOffset::east_opt(seconds)?)
            }
            None => now.with_timezone(&Local).fixed_offset(),
        };

        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}

impl IdeWorld for TideWorld {
    /// Upcasting. Returns a reference to this world as a trait object. See [`IdeWorld::upcast`].
    fn upcast(&self) -> &dyn World {
        self
    }
}

/// Initializes the standard Typst library with default styles.
///
/// Currently, sets the default text size.
fn library() -> Library {
    let mut lib = Library::default();
    lib.styles
        .set(TextElem::set_size(TextSize(Abs::pt(11.0).into())));
    //lib.styles.set(TextElem::set_font(FontList(...)));
    lib
}

impl Clone for TideWorld {
    fn clone(&self) -> TideWorld {
        Self {
            main: self.main,
            library: self.library.clone(),
            book: self.book.clone(),
            fonts: self.fonts.clone(),
            files: Files {
                assets: self.files.assets.clone(),
                sources: self.files.sources.clone(),
            },
        }
    }
}

impl Debug for TideWorld {
    /// Provides a formatted debug representation of the world,
    /// including the main file and currently loaded sources/assets.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let main_source = self.source(self.main);
        let content = if let Ok(main) = &main_source {
            main.text()
        } else {
            "NO CONTENT"
        };
        write!(
            f,
            "- main: {:?} (content: {})\n- sources: {:?}\n- assets: {:?}",
            main_source,
            content,
            self.files
                .sources
                .values()
                .map(|s| (s.id(), s.text()))
                .collect::<Vec<_>>(),
            self.files.assets
        )
    }
}

/// Internal file storage used by [`TideWorld`].
///
/// Manages both assets (binary files) and Typst sources (text files).
struct Files {
    /// Map of binary asset files, keyed by [`FileId`].
    assets: HashMap<FileId, Bytes>,
    /// Map of Typst source files, keyed by [`FileId`].
    sources: HashMap<FileId, Source>,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn init_world() -> TideWorld {
        let main_file_id = FileId::new(None, VirtualPath::new("main"));
        let mut world = TideWorld::new(main_file_id, None);
        let main = Source::new(main_file_id, String::from("= Hello World"));
        world.add_source(main_file_id, main);

        world
    }

    #[test]
    fn test_add_file() {
        let mut world = init_world();

        let new_file_id = FileId::new(None, VirtualPath::new("new_file.typ"));
        world.add_file(ImportedFile::TypstSource {
            file_id: new_file_id,
            source: Source::new(new_file_id, String::from("*test*")),
        });
        assert!(world.source(new_file_id).is_ok());
        assert_eq!(world.source(new_file_id).unwrap().text(), "*test*"); //unwrap() is ok because of the test above

        let asset_file_id = FileId::new(None, VirtualPath::new("fake_asset.svg"));
        world.add_file(ImportedFile::Asset {
            file_id: asset_file_id,
            bytes: Bytes::from_string("fake SVG"),
        });
        assert!(world.file(asset_file_id).is_ok());
        assert!(world.source(asset_file_id).is_err()); //because this is not a source file
        assert_eq!(world.file(asset_file_id).unwrap().len(), "fake SVG".len()); //unwrap() is ok because of the test above

        assert_eq!(world.files.assets.len(), 1);
        assert_eq!(world.files.sources.len(), 2);
    }

    #[test]
    fn test_remove_file() {
        let mut world = init_world();

        let new_file_id = FileId::new(None, VirtualPath::new("new_file.typ"));
        let asset_file_id = FileId::new(None, VirtualPath::new("fake_asset.svg"));
        world.add_source(
            new_file_id,
            Source::new(new_file_id, String::from("*test*")),
        );
        assert!(world.source(new_file_id).is_ok());
        world.add_asset(asset_file_id, Bytes::from_string("fake SVG"));
        assert!(world.file(asset_file_id).is_ok());

        world.remove_file(new_file_id);
        assert!(world.source(new_file_id).is_err());
        world.remove_file(asset_file_id);
        assert!(world.file(asset_file_id).is_err());
    }

    #[test]
    fn test_main() {
        let mut world = init_world();
        assert_eq!(world.main(), world.main);

        let new_file_id = FileId::new(None, VirtualPath::new("new_file.typ"));
        world.add_source(
            new_file_id,
            Source::new(new_file_id, String::from("*test*")),
        );
        world.change_main(new_file_id);
        assert_eq!(world.main(), new_file_id);
    }

    #[test]
    fn test_reload_source() {
        let mut world = init_world();
        assert!(world.source(world.main()).is_ok());
        assert_eq!(world.source(world.main()).unwrap().text(), "= Hello World"); //unwrap() is ok because of the test above

        world.reload_source_from_content(world.main(), &Content::with_text("= Text modified"));
        assert!(world.source(world.main()).is_ok());
        assert_eq!(
            world.source(world.main()).unwrap().text(),
            "= Text modified\n"
        ); //unwrap() is ok because of the test above
    }
}
