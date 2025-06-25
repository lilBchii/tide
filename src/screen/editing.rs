use super::component::{
    debug,
    file_tree::{self, FileTree},
    modal, pop_up,
    preview::Preview,
    status_bar::status_bar_view,
    toolbar::{self, editing_toolbar, open_url},
};

use crate::file_manager::export::template::export_template;
use crate::file_manager::export::ExportType;
use crate::file_manager::file::{
    cache_project, get_templates_path, load_file_dialog, save_file_dialog,
    save_file_disk, ProjectCache,
};
use crate::file_manager::import::{UploadType, ALL_TYPES, TEMPLATE};
use crate::screen::component::debug::DebugZone;
use crate::screen::component::modal::{FileModal, ProjectModal};
use crate::screen::component::pop_up::{PopUpElement, PopUpType};
use crate::widgets::vsplit::VSplit;
use crate::world::TideWorld;
use crate::{
    data::config::appearance::EditorConfig, file_manager::file::load_repo_dialog,
};
use crate::{
    data::config::appearance::HighlighterTheme, file_manager::export::pdf::export_pdf,
};
use crate::{editor, file_manager::export::errors::ExportError};
use crate::{
    editor::autocomplete::autocomplete, file_manager::file::delete_file_from_disk,
};
use crate::{editor::bindings::bindings, file_manager::import::load::load_repo};
use crate::{editor::buffer::Buffer, file_manager::import::load::load_file};
use crate::{
    file_manager::export::svg::{export_svg, preview_svg},
    font::EDITOR_FONT_FAMILY_NAME,
};
use iced::widget::center;
use iced::widget::text_editor::Edit;
use iced::Length::Fixed;
use iced::{
    advanced::svg::Handle,
    widget::{
        column, stack, svg,
        text_editor::{Action, Binding, Motion},
        Column, Scrollable, TextEditor,
    },
    Element, Font,
    Length::Fill,
    Shrink, Task,
};
use iced_aw::style::selection_list::primary;
use iced_aw::SelectionList;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::{collections::HashMap, fs, path::PathBuf};
use typst::ecow::EcoString;
use typst::syntax::{FileId, VirtualPath};
use typst::World;
use typst_ide::Completion;
use typst_pdf::PdfOptions;

/// Represents the current editing state of Tide.
///
/// Maintains buffers, file tree, project directory, preview rendering,
/// Typst world state, UI modals, and autocompletion context.
pub struct Editing {
    /// The currently selected [`Buffer`] and file.
    current: Current,
    /// All loaded file buffers indexed by their unique [`FileId`].
    buffers: HashMap<FileId, Buffer>,
    /// Root, absolute, path of the project (the project directory).
    current_dir: PathBuf,
    /// Current state of the document preview.
    preview: Preview,
    /// Interactive tree view of files in the current project.
    file_tree: FileTree,
    /// Represents the current Typst environment (source files, dependencies, etc), see [`TideWorld`].
    typst: TideWorld,
    /// Map of auto-paired characters for editor key bindings (e.g., `(` --> `)`).
    auto_pairs: HashMap<char, char>,
    /// Screen split positions between the file tree, editor, and preview.
    /// Tuple represents (file tree width, preview offset).
    split_at: (f32, f32),
    /// Autocompletion state including the available suggestions and cursor position.
    autocompletion_ctx: AutocompletionContext,
    /// Pop-up element currently displayed over the UI.
    pop_up: Option<PopUpElement>,
    /// Debug UI zone (e.g. diagnostics, logs).
    debug: Option<DebugZone>,
    /// Modal window for creating a file.
    file_modal: FileModal,
    /// Modal window for creating a new project.
    project_modal: ProjectModal,
    theme: HighlighterTheme,
}

impl Editing {
    /// Creates a new [`Editing`] instance with the given configuration and directory.
    ///
    /// Initializes the Typst world, file tree, and UI context.
    pub fn new(
        config: EditorConfig,
        current_dir: PathBuf,
    ) -> Self {
        println!("{:?}", config.colors);
        Self {
            current: Current::empty(),
            buffers: HashMap::new(),
            current_dir: current_dir.clone(),
            preview: Preview::new(),
            file_tree: FileTree::new(current_dir.to_path_buf()),
            typst: init_world(),
            auto_pairs: config.auto_pairs,
            split_at: (250.0, 800.0),
            autocompletion_ctx: AutocompletionContext::new(),
            pop_up: None,
            debug: None,
            file_modal: FileModal::new(current_dir.to_path_buf()),
            project_modal: ProjectModal::new(),
            theme: config.colors,
        }
    }

    /// Returns a reference to the current [`Buffer`] in use.
    fn current_buffer(&self) -> &Buffer {
        &self.current.buffer
    }

    /// Returns the currently active [`FileId`] if available.
    fn current_file_id(&self) -> Option<FileId> {
        self.current.file_id
    }

    /// Updates the Typst source and in-memory buffer for the given file.
    ///
    /// Replaces the buffer's content and reloads the source in the Typst world.
    fn update_source(
        &mut self,
        id: FileId,
        buffer: Buffer,
    ) {
        println!("updating source for {:?}", id);
        self.typst.reload_source_from_content(id, &buffer.content);
        if let Some(buf) = self.buffers.get_mut(&id) {
            buf.content = buffer.content;
        }
    }

    /// Returns `true` if all open buffers have been saved.
    fn all_saved(&self) -> bool {
        self.buffers.iter().all(|(_, buffer)| buffer.is_saved)
    }

    /// Closes the buffer associated with the given [`FileId`].
    fn close_buffer(
        &mut self,
        id: FileId,
    ) {
        self.buffers.remove(&id);
    }

    /// Deletes a file both from disk and from the Typst world.
    /// Also removes the file from the buffer and refreshes the file tree.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file could not be deleted.
    fn delete_file(
        &mut self,
        id: FileId,
    ) -> Result<(), Error> {
        //remove file from disk
        delete_file_from_disk(id, self.current_dir.clone())?;
        //remove file from buffer
        self.close_buffer(id);
        //remove file from Typst world
        self.typst.remove_file(id);
        //remove file from file tree
        self.file_tree.refresh();
        Ok(())
    }

    /// Uploads a file from disk into the Typst environment and the file tree.
    /// The file is copied to the project directory and loaded into the editor.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be read or written.
    fn upload_file(
        &mut self,
        path: &PathBuf,
    ) -> Result<(), Error> {
        //retrieve content and file name
        let file_content = fs::read(path)?;
        let file_path = self
            .current_dir
            .join(path.file_name().ok_or(Error::from(ErrorKind::Other))?);

        //copy
        fs::write(&file_path, file_content)?;

        //load in the editor and Typst
        let imported_file = load_file(&file_path, &self.current_dir)?;
        self.typst.add_file(imported_file);
        self.file_tree.add_file(&file_path);

        Ok(())
    }

    /// Attempts to create a new empty file at the specified path and uploads it into the project.
    fn create_file(
        &mut self,
        path: PathBuf,
    ) {
        if let Err(e) = fs::File::create(&path) {
            eprintln!("Error when creating the file : {}", e);
        }
        println!("new file created:\n{:?}", path);
        if let Err(e) = self.upload_file(&path.clone()) {
            eprintln!("Error when uploading: {}", e);
        }
    }

    /// Returns the main [`Element<Message>`] view for the editing screen.
    ///
    /// Composes the file tree, text editor, preview (if available), status bar,
    /// and optional modals or pop-ups.
    pub fn view(&self) -> Element<Message> {
        //let tool_bar = self.tool_bar.view().map(Message::ToolBar);
        let tool_bar =
            editing_toolbar(Some(self.typst.main().vpath())).map(Message::ToolBar);
        let editor = TextEditor::new(&self.current_buffer().content)
            .on_action(Message::ActionPerformed)
            .placeholder("Insert text here or open a new file")
            .key_binding(|key_press| {
                if let Some(ref text) = key_press.text {
                    if let Some(actual_char) = text.chars().nth(0) {
                        if self.auto_pairs.contains_key(&actual_char) {
                            // enclose the selection with the auto pair characters
                            if let Some(selection) =
                                self.current_buffer().content.selection()
                            {
                                let mut seq: Vec<Binding<Message>> = Vec::new();
                                seq.push(Binding::Insert(actual_char));
                                for c in selection.chars() {
                                    seq.push(Binding::Insert(c));
                                }
                                seq.push(Binding::Insert(
                                    *self.auto_pairs.get(&actual_char).unwrap(),
                                ));
                                Some(Binding::Sequence(seq))
                            } else {
                                Some(Binding::Sequence(vec![
                                    Binding::Insert(actual_char),
                                    Binding::Insert(
                                        *self.auto_pairs.get(&actual_char).unwrap(),
                                    ),
                                    Binding::Move(Motion::Left),
                                ]))
                            }
                        } else {
                            bindings(key_press)
                        }
                    } else {
                        bindings(key_press)
                    }
                } else {
                    bindings(key_press)
                }
            })
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .height(Fill)
            .highlight_with::<editor::highlighter::Highlighter>(
                editor::highlighter::Settings {
                    theme: self.theme.clone(),
                    extension: "typ".to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            )
            .font(Font::with_name(EDITOR_FONT_FAMILY_NAME));

        let cursor_pos = self.current.buffer.content.cursor_position();

        let (split_left, split_right) = self.split_at;
        let (tree, _h) = self.file_tree.view();

        let file_tree = Scrollable::new(tree.map(Message::FileTree));

        let mut edit_col = Column::new().push(editor);
        if let Some(debug) = &self.debug {
            let stack_w_debug = edit_col.push(debug.view().map(Message::DebugSpace));
            edit_col = stack_w_debug;
        } //debug

        let mut main_screen = VSplit::new(file_tree, edit_col)
            .strategy(crate::widgets::vsplit::Strategy::Left)
            .split_at(split_left)
            .on_resize(Message::ResizeTree); //VSplit without preview

        if let Some(svg_handles) = &self.preview.handle {
            let mut svg_pages = vec![];
            for page in svg_handles.to_owned() {
                svg_pages.push(svg(page).into());
            }
            let preview =
                Scrollable::new(Column::with_children(svg_pages).spacing(15).padding(15))
                    .width(Fill)
                    .height(Fill);
            main_screen = VSplit::new(main_screen, preview)
                .strategy(crate::widgets::vsplit::Strategy::Left)
                .split_at(split_right)
                .on_resize(Message::ResizePreview);
        } //VSplit with preview

        //--------//

        let status_bar = status_bar_view(
            cursor_pos,
            match self.current.file_id {
                Some(id) => id.vpath().as_rootless_path().to_string_lossy().to_string(),
                None => "No file selected".to_string(),
            },
            self.current.buffer.is_saved,
        ); //status bar

        let screen = column![tool_bar, main_screen, status_bar];

        if let Some(completions) = &self.autocompletion_ctx.completions {
            let selection = center(
                SelectionList::new_with(
                    completions,
                    Message::ApplyAutocomplete,
                    12.0,
                    5.0,
                    primary,
                    None,
                    Font::default(),
                )
                .width(Shrink)
                .height(Fixed(100.0)),
            );
            return stack![screen, selection].into();
        } //autocomplete

        if let Some(pop_up) = &self.pop_up {
            return stack![screen, pop_up.view().map(Message::PopUp)].into();
        } //pop-up

        if self.file_modal.visible {
            return stack![screen, self.file_modal.view().map(Message::FileModal)].into();
        } //"new file" modal

        if self.project_modal.visible {
            return stack![screen, self.project_modal.view().map(Message::ProjectModal)]
                .into();
        } //"new project" modal

        screen.into() //default
    }

    /// Updates the editing state in response to a [`Message`] input.
    pub fn update(
        &mut self,
        message: Message,
    ) -> Task<Message> {
        match message {
            Message::CachedProject(main) => {
                println!("Project {:?} cached!", self.current_dir);
                if let Some(main_path) = main {
                    println!("Main retrieved from cache, change it: {:?}", main_path);
                    return Task::done(Message::FileTree(
                        file_tree::Message::ChangeMainFile(main_path),
                    ));
                }
                Task::none()
            }
            Message::PopUp(message) => {
                match message {
                    pop_up::Message::ShowPopUp(pop_up_element) => {
                        if self.pop_up.is_none() {
                            self.pop_up = Some(pop_up_element);
                        } else {
                            println!("other pop-up..."); //todo: if there are several pop-ups in a row, put them in a queue!
                        }
                        Task::none()
                    }
                    pop_up::Message::HidePopUp => {
                        self.pop_up = None;
                        Task::none()
                    }
                    pop_up::Message::DeleteFile(id) => match self.delete_file(id) {
                        Ok(_) => Task::done(Message::PopUp(pop_up::Message::HidePopUp)),
                        Err(err) => Task::done(Message::PopUp(
                            pop_up::Message::ShowPopUp(PopUpElement::new(
                                PopUpType::Error,
                                String::from("Can't delete file!"),
                                err.to_string(),
                            )),
                        )),
                    },
                }
            }
            Message::DebugSpace(message) => match message {
                debug::Message::ShowErrors(debug_zone) => {
                    self.debug = Some(debug_zone);
                    Task::none()
                }
                debug::Message::HideErrors => {
                    self.debug = None;
                    Task::none()
                }
            },

            Message::ApplyAutocomplete(selected, completion) => {
                self.autocompletion_ctx.completions = None;
                println!("selected {} completion: {:?}", selected, completion);

                Task::done(Message::ActionPerformed(Action::Edit(Edit::Paste(
                    completion.apply_to(&self.autocompletion_ctx),
                ))))
            }
            Message::ShowAutocomplete(pos, completions) => {
                if !completions.is_empty() {
                    self.autocompletion_ctx.pos = pos;
                    self.autocompletion_ctx.completions = Some(
                        completions
                            .into_iter()
                            .map(DisplayableCompletion::from)
                            .collect(),
                    );
                }

                Task::none()
            }
            Message::Autocomplete => {
                if let Some(current_file_id) = self.current_file_id() {
                    self.update_source(current_file_id, self.current_buffer().clone());
                    let (line, shift) = self.current_buffer().content.cursor_position();
                    if let Ok(source) = self.typst.source(self.current_file_id().unwrap())
                    {
                        let index = source.line_column_to_byte(line, shift);
                        println!("{:?}", self.typst);
                        println!(
                            "autocompletion for {:?}/len:{}",
                            source,
                            source.len_bytes()
                        );
                        println!(
                            "line: {} / shift: {} | index: {:?}",
                            line, shift, index
                        );
                        if let Some(cursor_index) = index {
                            let Some((pos, completions)) =
                                autocomplete(&self.typst, &source, cursor_index)
                            else {
                                return Task::none();
                            };
                            self.autocompletion_ctx.cursor = cursor_index;
                            return Task::done(Message::ShowAutocomplete(
                                pos, completions,
                            ));
                        }
                    }
                }
                Task::none()
            }
            Message::ResizeTree(split_at) => {
                self.split_at.0 = split_at.clamp(100.0, self.split_at.1.min(500.0));
                Task::none()
            }
            Message::ResizePreview(split_at) => {
                self.split_at.1 = split_at.clamp(self.split_at.0.max(200.0), 1500.0);
                Task::none()
            }
            Message::ActionPerformed(action) => {
                let is_edit = action.is_edit();
                self.current.buffer.content.perform(action);
                if self.current.buffer.is_saved && is_edit {
                    self.current.buffer.is_saved = false;
                }
                Task::none()
            }
            Message::SvgGenerated(result) => {
                println!("async: SVG generated");
                match result {
                    Ok(svg) => Task::perform(
                        async move {
                            let mut svg_handles: Vec<Handle> = vec![];
                            for content in svg {
                                svg_handles
                                    .push(Handle::from_memory(content.into_bytes()));
                            }
                            svg_handles
                        },
                        Message::PreviewLoaded,
                    ),
                    Err(err) => Task::done(Message::DebugSpace(
                        debug::Message::ShowErrors(DebugZone::new(err.to_string())),
                    )),
                }
            }
            Message::PreviewLoaded(svg_handles) => {
                println!("async: preview loaded");
                self.preview.handle = Some(svg_handles);
                Task::done(Message::DebugSpace(debug::Message::HideErrors))
            }
            Message::ToolBar(message) => {
                match message {
                    toolbar::Message::StartFromTemplate => {
                        let templates_path =
                            get_templates_path().unwrap_or(self.current_dir.to_owned());
                        if let Some(template) =
                            load_file_dialog(&templates_path, "Typst template", &TEMPLATE)
                        {
                            self.project_modal.require_template(template);
                            return Task::done(Message::ToolBar(
                                toolbar::Message::NewProject,
                            ));
                        }
                        Task::done(Message::PopUp(pop_up::Message::ShowPopUp(
                            PopUpElement::new(
                                PopUpType::Warning,
                                String::from("Project creation cancelled!"),
                                String::from(
                                    "We can't create your project without template.",
                                ),
                            ),
                        )))
                    }
                    toolbar::Message::ForcePreview => {
                        if let Some(id) = self.current_file_id() {
                            self.update_source(id, self.current_buffer().clone());
                        }
                        Task::perform(
                            preview_svg(self.typst.clone()),
                            Message::SvgGenerated,
                        )
                    }
                    toolbar::Message::SaveFile(update) => {
                        if let Some(id) = self.current_file_id() {
                            if update {
                                self.update_source(id, self.current_buffer().clone());
                            }
                            return Task::perform(
                                save_file_disk(
                                    id,
                                    self.current_buffer().clone(),
                                    self.current_dir.clone(),
                                ), //it should be the Source file...
                                |result| {
                                    Message::ToolBar(toolbar::Message::FileSaved(result))
                                },
                            );
                        }

                        Task::none()
                    }
                    toolbar::Message::FileSaved(result) => match result {
                        Ok(path) => {
                            println!("file saved at {:?}", path);
                            self.current.buffer.is_saved = true;
                            Task::none()
                        }
                        Err(err) => Task::done(Message::PopUp(
                            pop_up::Message::ShowPopUp(PopUpElement::new(
                                PopUpType::Error,
                                String::from("File not saved!"),
                                err.to_string(),
                            )),
                        )),
                    },
                    toolbar::Message::Upload(upload_type, import_path) => {
                        let import_path =
                            import_path.unwrap_or(self.current_dir.to_owned());
                        if let Some(path) = match upload_type {
                            UploadType::All => load_file_dialog(
                                &import_path,
                                "Typst files or assets",
                                &ALL_TYPES,
                            ),
                            UploadType::Template => load_file_dialog(
                                &import_path,
                                "Typst template",
                                &TEMPLATE,
                            ),
                        } {
                            if let Some(file_name) = path.file_name() {
                                return if !self.current_dir.join(file_name).exists() {
                                    match self.upload_file(&path) {
                                        Ok(_) => Task::done(Message::ToolBar(
                                            toolbar::Message::FileImported(Ok(path)),
                                        )),
                                        Err(err) => Task::done(Message::PopUp(
                                            pop_up::Message::ShowPopUp(
                                                PopUpElement::new(
                                                    PopUpType::Error,
                                                    String::from("Can't upload file!"),
                                                    err.to_string(),
                                                ),
                                            ),
                                        )),
                                    }
                                } else {
                                    Task::done(Message::PopUp(pop_up::Message::ShowPopUp(
                                    PopUpElement::new(
                                        PopUpType::Error,
                                        String::from("Can't upload file"),
                                        String::from("File with the same name already exists in the project.")))))
                                };
                            }
                        }
                        Task::none()
                    }
                    toolbar::Message::FileImported(result) => match result {
                        Ok(path) => {
                            println!("file imported: {:?}", path); //todo: normal pop-up
                            Task::none()
                        }
                        Err(err) => Task::done(Message::PopUp(
                            pop_up::Message::ShowPopUp(PopUpElement::new(
                                PopUpType::Error,
                                String::from("File not imported!"),
                                err.to_string(),
                            )),
                        )),
                    },
                    toolbar::Message::OpenProject(path_option, main) => {
                        let path = match path_option {
                            Some(import_path) => import_path,
                            None => {
                                let dialog_path = load_repo_dialog();
                                match dialog_path {
                                    Some(import_path) => import_path,
                                    None => return Task::none(),
                                }
                            }
                        };
                        // change current dir, reset the file tree, reset "new file" modal
                        self.current_dir = path.clone();
                        self.file_tree = FileTree::new(path.to_path_buf());
                        self.file_modal = FileModal::new(path.to_path_buf());

                        // load files in TideWorld
                        if let Ok(imported_files) = load_repo(&path, &self.current_dir) {
                            for imported_file in imported_files {
                                self.typst.add_file(imported_file);
                            }
                        }

                        Task::perform(
                            cache_project(ProjectCache::new(
                                self.current_dir.to_owned(),
                                main.to_owned(),
                            )),
                            move |_| Message::CachedProject(main.to_owned()),
                        )
                    }
                    // TODO: write the real open file
                    toolbar::Message::OpenFile => {
                        /*
                        let path = load_file_dialog("*", &["typ"]);
                        if let Some(import_path) = path {
                            // set the parent dir of the file as current dir
                            let import_path = import_path.parent().unwrap().to_path_buf();
                            self.current_dir = import_path.clone();
                            self.file_tree = Dir::new(import_path.clone());
                            // load file in TideWorld
                            if let Ok(imported_file) = load_file(&import_path) {
                                self.typst.add_file(imported_file);
                            }
                            Task::done(Message::ToolBar(toolbar::Message::FileImported(Ok(
                                import_path,
                            ))))
                        } else {
                            Task::none()
                        }
                        */
                        Task::none()
                    }
                    toolbar::Message::Export(export_type) => {
                        match export_type {
                            ExportType::PDF => {
                                let path = save_file_dialog("pdf", &["pdf"]);
                                if let Some(export_path) = path {
                                    Task::perform(
                                        export_pdf(
                                            self.typst.clone(),
                                            export_path,
                                            PdfOptions::default(),
                                        ),
                                        |result| {
                                            Message::ToolBar(
                                                toolbar::Message::ProjectExported(result),
                                            )
                                        },
                                    )
                                } else {
                                    Task::none() //abort or error
                                }
                            }
                            ExportType::SVG => {
                                let path = save_file_dialog("svg", &["svg"]);
                                if let Some(export_path) = path {
                                    Task::perform(
                                        export_svg(self.typst.clone(), export_path),
                                        |result| {
                                            Message::ToolBar(
                                                toolbar::Message::ProjectExported(result),
                                            )
                                        },
                                    )
                                } else {
                                    Task::none() //abort or error
                                }
                            }
                            ExportType::Template => {
                                let export_path = get_templates_path()
                                    .unwrap_or(self.current_dir.to_owned());
                                let file_path = self.file_tree.selected_path.to_owned();
                                if let Some(current_path) = file_path {
                                    return Task::perform(
                                        export_template(current_path, export_path),
                                        |result| {
                                            Message::ToolBar(
                                                toolbar::Message::ProjectExported(result),
                                            )
                                        },
                                    );
                                }
                                Task::done(Message::PopUp(pop_up::Message::ShowPopUp(
                                PopUpElement::new(
                                    PopUpType::Warning,
                                    String::from("No file selected!"),
                                    String::from("Please select a file to export it as template."),
                                ),
                            )))
                            }
                        }
                    }
                    toolbar::Message::AddTemplate => {
                        Task::done(Message::ToolBar(toolbar::Message::Upload(
                            UploadType::Template,
                            get_templates_path(),
                        )))
                    }
                    toolbar::Message::ProjectExported(result) => match result {
                        //TODO: Status bar
                        Ok(_path) => {
                            println!("Project exported!");
                            Task::none()
                        }
                        Err(err) => Task::done(Message::PopUp(
                            pop_up::Message::ShowPopUp(PopUpElement::new(
                                PopUpType::Error,
                                String::from("Project not exported!"),
                                err.to_string(),
                            )),
                        )),
                    },
                    toolbar::Message::Universe => {
                        open_url("https://typst.app/universe/");
                        Task::none()
                    }
                    toolbar::Message::Help => {
                        open_url("https://typst.app/docs/");
                        Task::none()
                    }
                    toolbar::Message::NewFile => {
                        self.file_modal.show();
                        Task::none()
                    }
                    toolbar::Message::NewProject => {
                        self.project_modal.show();
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
            Message::FileTree(message) => {
                match message {
                    file_tree::Message::DirClick(path) => {
                        self.file_tree.fold(&path);
                    }
                    file_tree::Message::ChangeCurrentFile(path) => {
                        if let Some(id) =
                            TideWorld::id_from_path(&path, &self.current_dir)
                        {
                            if let Some(id) = self.current_file_id() {
                                self.update_source(id, self.current_buffer().clone());
                            }
                            if let Some(buffer) = self.buffers.get(&id) {
                                self.current.set(buffer.clone(), id);
                                self.file_tree.change_selected(&path);
                            } else {
                                return match Buffer::from_path(&path) {
                                    Ok(buffer) => {
                                        self.buffers.insert(id, buffer);
                                        println!("current buffers: {:?}", self.buffers);
                                        Task::done(Message::FileTree(
                                            file_tree::Message::ChangeCurrentFile(path),
                                        ))
                                    }
                                    Err(error) => {
                                        println!("can't add buffer: {:?}", error);
                                        Task::done(Message::PopUp(pop_up::Message::ShowPopUp(
                                            PopUpElement::new(
                                                PopUpType::Warning,
                                                String::from("Can't open file"),
                                                format!("We can't open and preview this kind of file yet.\n{:?}", path)
                                            )))
                                        )
                                    }
                                };
                            }
                        }
                    }
                    file_tree::Message::ChangeMainFile(path) => {
                        match TideWorld::id_from_path(&path, &self.current_dir) {
                            Some(new_main_id) => {
                                println!("new main file: {:?}", new_main_id);
                                //self.tool_bar.main_file_text = format!("{:?}", new_main_id.vpath());
                                self.typst.change_main(new_main_id);
                                self.file_tree.change_main(&path);
                                return Task::batch([
                                    Task::perform(
                                        cache_project(ProjectCache::new(
                                            self.current_dir.to_owned(),
                                            Some(path),
                                        )),
                                        move |_| Message::CachedProject(None),
                                    ),
                                    Task::done(Message::FileTree(
                                        file_tree::Message::FileExit,
                                    )),
                                ]);
                            }
                            None => {
                                return Task::done(Message::PopUp(pop_up::Message::ShowPopUp(
                                    PopUpElement::new(
                                        PopUpType::Warning,
                                        String::from("Can't change main file"),
                                        format!(
                                            "File not found in the current project.\n{:?}",
                                            path
                                        ),
                                    ),
                                )));
                            }
                        }
                    }
                    file_tree::Message::FileExit => self.file_tree.change_clicked(None),
                    file_tree::Message::FileClick(path) => {
                        self.file_tree.change_clicked(Some(path));
                    }
                    file_tree::Message::DeleteFile(path) => {
                        if let Some(id) =
                            TideWorld::id_from_path(&path, &self.current_dir)
                        {
                            return Task::done(Message::PopUp(
                                pop_up::Message::ShowPopUp(PopUpElement::new(
                                    PopUpType::Confirm(id),
                                    "Want to delete file".to_string(),
                                    path.to_string_lossy().to_string(),
                                )),
                            ));
                        }
                        return Task::done(Message::FileTree(
                            file_tree::Message::FileExit,
                        ));
                    }
                }
                Task::none()
            }
            Message::FileModal(message) => self.file_modal.update(message),
            Message::ProjectModal(message) => {
                self.project_modal.update(message).map(Message::ToolBar)
            }
            Message::CreateFile(path) => {
                self.create_file(path);
                Task::none()
            }
        }
    }
}

/// Initializes the [`TideWorld`] with a temporary, fake, main file.
fn init_world() -> TideWorld {
    let main_id = FileId::new_fake(VirtualPath::new("main.typ"));
    TideWorld::new(main_id, None)
}

/// Tracks the currently selected buffer and file.
struct Current {
    /// Current buffer.
    buffer: Buffer,
    /// Associated [`FileId`], if it exists.
    file_id: Option<FileId>,
}

impl Current {
    /// Creates a new [`Current`] from the given buffer and file ID.
    fn new(
        buffer: Buffer,
        file_id: Option<FileId>,
    ) -> Self {
        Self { buffer, file_id }
    }

    /// Returns an empty [`Current`] state with no buffer loaded.
    fn empty() -> Self {
        Self {
            buffer: Buffer::new(),
            file_id: None,
        }
    }

    /// Sets the current buffer and file ID to the provided values.
    fn set(
        &mut self,
        buffer: Buffer,
        file_id: FileId,
    ) {
        self.buffer = buffer;
        self.file_id = Some(file_id);
    }
}

/// Stores the current autocompletion state, including available completions,
/// cursor position, and completion offset.
struct AutocompletionContext {
    /// List of displayable completions.
    completions: Option<Vec<DisplayableCompletion>>,
    /// Current cursor position in the editor.
    cursor: usize,
    /// Current completion position in the editor.
    pos: usize,
}

impl AutocompletionContext {
    /// Creates a new empty [`AutocompletionContext`].
    fn new() -> Self {
        Self {
            completions: None,
            cursor: 0,
            pos: 0,
        }
    }
}

//...because Completion need to implement Display, Eq, Hash (for SelectionList)
/// A wrapper around [`Completion`] that implements display and hashing,
/// making it usable in a selection list.
#[derive(Clone, Debug)]
struct DisplayableCompletion {
    completion: Completion,
}

impl DisplayableCompletion {
    /// Constructs a [`DisplayableCompletion`] from a [`Completion`] instance.
    fn from(completion: Completion) -> Self {
        Self { completion }
    }

    /// Applies the completion to the current context, trimming the already typed prefix.
    ///
    /// Returns the resulting text as an [`Arc<String>`].
    //for example:
    //"#ima" - cursor is at position 4, but completion occurs at position 1
    //therefore, we should remove the first 0, 1, ..., n = cursor-pos characters of the apply/label
    //here, n = 4-1, we remove characters in range 0..(<)3 of "image(${})"
    //this leaves the sub-chain "ge(${})"
    fn apply_to(
        &self,
        ctx: &AutocompletionContext,
    ) -> Arc<String> {
        let diff = ctx.cursor - ctx.pos;
        let label = self.completion.label.to_owned();
        let apply = self.completion.apply.to_owned().unwrap_or(label);
        let complete = String::from(apply.strip_prefix(&apply[0..diff]).unwrap_or(""));
        Arc::from(complete)
    }
}

impl Display for DisplayableCompletion {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        let details = self.completion.detail.clone();
        write!(
            f,
            "({:?}) {} - {}",
            self.completion.kind,
            self.completion.label,
            details.unwrap_or(EcoString::from("no description..."))
        )
    }
}

impl PartialEq<Self> for DisplayableCompletion {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.completion.label == other.completion.label
    }
}

impl Eq for DisplayableCompletion {}

impl Hash for DisplayableCompletion {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.completion.label.hash(state);
    }
}

/// Messages handled by the _Editing_ view.
#[derive(Debug, Clone)]
pub enum Message {
    /// A message emitted by the toolbar (e.g., button click or input).
    ToolBar(toolbar::Message),
    /// A text action performed inside the editor (e.g., typing or pasting).
    ActionPerformed(Action),
    /// A message emitted from the file tree component (e.g., file clicked or deleted).
    FileTree(file_tree::Message),
    /// Resizes the file tree area to the given width.
    ResizeTree(f32),
    /// Resizes the preview area to the given width.
    ResizePreview(f32),
    /// Result of an SVG export operation for the current document.
    SvgGenerated(Result<Vec<String>, ExportError>),
    /// Loaded preview images (SVG handles) to be rendered on screen.
    PreviewLoaded(Vec<Handle>),
    /// Triggers the autocompletion logic based on current cursor position.
    Autocomplete,
    /// Displays the autocompletion menu with the given suggestions.
    ///
    /// First parameter is the cursor position. Second is the list of [`Completion`]s.
    ShowAutocomplete(usize, Vec<Completion>),
    /// Applies the selected autocompletion suggestion at the specified position.
    ApplyAutocomplete(usize, DisplayableCompletion),
    /// A message emitted by the currently displayed pop-up window.
    PopUp(pop_up::Message),
    /// A message emitted by the debug zone (e.g., toggling view or filtering logs).
    DebugSpace(debug::Message),
    /// Indicates a cached project was loaded or changed.
    ///
    /// `None` means the main file doesn't have to be reset.
    CachedProject(Option<PathBuf>),
    /// Creates a new file in the current project.
    CreateFile(PathBuf),
    /// A message emitted by the "new file" modal.
    FileModal(modal::Message),
    /// A message emitted by the "new project" modal.
    ProjectModal(modal::Message),
}

#[cfg(test)]
mod test {
    use super::*;
    use typst::syntax::Source;

    const MAIN_FILE_NAME: &str = "main.typ";
    const OTHER_FILE_NAME: &str = "other.typ";
    const TEST_CONTENT: &str = "this is a test";
    const TEST_PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

    fn create_editing() -> Editing {
        let path = PathBuf::from(TEST_PROJECT_ROOT);
        let config = EditorConfig::default();
        Editing::new(config, path)
    }

    fn create_file_id() -> FileId {
        FileId::new_fake(VirtualPath::new(OTHER_FILE_NAME))
    }

    fn main_file_id() -> FileId {
        FileId::new_fake(VirtualPath::new(MAIN_FILE_NAME))
    }

    fn create_buffer() -> Buffer {
        Buffer::new()
    }

    #[test]
    fn test_close_buffer() {
        let mut editing = create_editing();
        let file_id = create_file_id();
        let buffer = create_buffer();
        editing.buffers.insert(file_id, buffer);
        editing.close_buffer(file_id);
        println!("{:?}", editing.buffers);
        assert!(editing.buffers.is_empty())
    }

    // #[test]
    // fn test_delete_file() {
    //     let text = "Hello, World!";
    //     let path = "assets/testis/file.typ";
    //     let _ = fs::write(path, text);
    //     let file_id = FileId::new_fake(VirtualPath::new(path));
    //     let mut editing = create_editing();
    //     editing
    //         .typst
    //         .add_source(file_id, Source::new(file_id, text.to_string()));
    //     assert!(editing.typst.source(file_id).is_ok());
    //     let _ = editing.delete_file(file_id);
    //     assert!(editing.buffers.is_empty());
    //     assert!(editing.typst.source(file_id).is_err());
    //     assert!(fs::read(path).is_err());
    // }

    #[test]
    fn test_pop_up() {
        let pop_up_el = PopUpElement::new(
            PopUpType::Warning,
            "title".to_string(),
            "there is an error".to_string(),
        );
        let mut editing = create_editing();
        let _task = editing.update(Message::PopUp(pop_up::Message::ShowPopUp(
            pop_up_el.clone(),
        )));
        assert_eq!(editing.pop_up, Some(pop_up_el));
        let _task = editing.update(Message::PopUp(pop_up::Message::HidePopUp));
        assert!(editing.pop_up.is_none());
    }

    #[test]
    fn test_editor_action_performed() {
        let mut editing = create_editing();
        let _task =
            editing.update(Message::ActionPerformed(Action::Edit(Edit::Insert('a'))));
        assert_eq!(editing.current.buffer.content.text(), "a\n");
    }

    #[test]
    fn test_change_main_file() {
        let mut editing = create_editing();
        let file_id = create_file_id();
        editing
            .typst
            .add_source(file_id, Source::new(file_id, String::from(TEST_CONTENT)));
        let _task =
            editing.update(Message::FileTree(file_tree::Message::ChangeMainFile(
                PathBuf::from(TEST_PROJECT_ROOT).join(OTHER_FILE_NAME),
            )));

        assert_eq!(
            editing.typst.main().vpath().as_rootless_path(),
            file_id.vpath().as_rootless_path()
        );
    }

    #[test]
    fn test_modal() {
        let mut editing = create_editing();
        let _task = editing.update(Message::FileModal(modal::Message::Cancel));
        assert_eq!(editing.file_modal.file_name, String::new());
        assert!(!editing.file_modal.visible);
    }
}
