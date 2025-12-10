use std::path::PathBuf;

use crate::{
    data::style::{
        button::toolbar_button,
        file_tree::{main_style, selected_file},
        tooltip::tooltip_box,
    },
    icon,
};
use iced::{
    padding,
    widget::{
        button, column, container, mouse_area, row, rule, space, text, text::Wrapping,
        tooltip, Text,
    },
    Alignment, Element, Length,
};

const HEIGHT: f32 = 20.0;
const INDENT: f32 = 10.0;
const SPACING: f32 = 5.0;
const ICON_BUTTON_SIZE: f32 = 24.0;

/// Represents messages triggered by user interactions with the file tree.
#[derive(Clone, Debug)]
pub enum Message {
    /// Set the given file as the main file for the project.
    ChangeMainFile(PathBuf),
    /// Toggle a directory's open/closed state on click.
    DirClick(PathBuf),
    /// Change the currently selected file to the one at the given path.
    ChangeCurrentFile(PathBuf),
    /// Handle a file being right-clicked.
    FileClick(PathBuf),
    /// Delete the file at the specified path.
    DeleteFile(PathBuf),
    /// Triggered when the mouse exits a file item.
    FileExit,
}

/// Represents the interactive file tree UI component of Tide.
///
/// Stores and manages state related to the current directory structure,
/// the selected and clicked files, and the main Typst file.
pub struct FileTree {
    /// Internal representation of the file system.
    file_system: Dir,
    /// Path to the file marked as the main Typst file.
    main_path: Option<PathBuf>,
    /// Path to the currently selected file in the UI.
    pub selected_path: Option<PathBuf>,
    /// Path to the file that was most recently clicked.
    clicked_path: Option<PathBuf>,
}

impl FileTree {
    /// Creates a new [`FileTree`] rooted at the given directory path.
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            file_system: Dir::new(root_path),
            main_path: None,
            selected_path: None,
            clicked_path: None,
        }
    }

    /// Builds and returns the file tree view as an [`Element`].
    ///
    /// Returns the UI element and its height.
    pub fn view(&self) -> (Element<'_, Message>, f32) {
        self.file_system
            .view(&self.clicked_path, &self.main_path, &self.selected_path)
    }

    /// Adds a file to the file tree.
    ///
    /// The file is inserted into the internal list of files and sorted alphabetically.
    pub fn add_file(
        &mut self,
        path: &PathBuf,
    ) {
        let file = File::new(path);
        if let Some(mut files) = self.file_system.files.clone() {
            files.push(file);
            files.sort_by(|a, b| a.name.cmp(&b.name));
            self.file_system.files = Some(files.to_vec())
        }
    }

    /// Refreshes file entries in the tree.
    ///
    /// Useful after a file has been deleted or modified externally.
    pub fn refresh(&mut self) {
        self.file_system.reload_files();
    }

    /// Toggles open/closed state of a directory node.
    pub fn fold(
        &mut self,
        path: &PathBuf,
    ) {
        self.file_system.update(path);
    }

    /// Changes the main Typst file path.
    ///
    /// If the path is already the main file, no change is made.
    pub fn change_main(
        &mut self,
        path: &PathBuf,
    ) {
        match self.main_path.clone() {
            Some(current_path) if current_path != *path => {
                self.main_path = Some(path.clone());
            }
            None => {
                self.main_path = Some(path.clone());
            }
            _ => {}
        }
    }

    /// Changes the currently selected file.
    pub fn change_selected(
        &mut self,
        path: &PathBuf,
    ) {
        match self.selected_path.clone() {
            Some(current_path) if current_path != *path => {
                self.selected_path = Some(path.clone());
            }
            None => {
                self.selected_path = Some(path.clone());
            }
            _ => {}
        }
    }

    /// Changes the last clicked file (used for file actions).
    pub fn change_clicked(
        &mut self,
        path: Option<PathBuf>,
    ) {
        match path {
            None => self.clicked_path = None,
            Some(path) => match self.clicked_path.clone() {
                Some(current_path) if current_path != *path => {
                    self.clicked_path = Some(path.clone());
                }
                None => {
                    self.clicked_path = Some(path.clone());
                }
                _ => {}
            },
        }
    }
}

/// Represents a directory node in the file system tree.
pub struct Dir {
    /// The root path.
    path: PathBuf,
    /// The name of the [Dir].
    name: String,
    /// The subdirectories of the [Dir].
    dirs: Option<Vec<Dir>>,
    /// The files stored within the [Dir].
    files: Option<Vec<File>>,
    /// The expansion state.
    is_open: bool,
}

impl Dir {
    /// Creates a new [`Dir`] from the given path.
    ///
    /// Initializes the name and path. Child files and directories are not loaded.
    fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap();

        Self {
            path: path.clone(),
            name: name.into(),
            dirs: None,
            files: None,
            is_open: false,
        }
    }

    /// Updates the expansion state of the directory at the specified path.
    ///
    /// If the path matches this node, toggles its open state and loads children.
    fn update(
        &mut self,
        path: &PathBuf,
    ) {
        if &*self.path == path {
            self.is_open ^= true;

            if self.is_open {
                if self.dirs.is_none() {
                    self.dirs = Some(self.init_dirs());
                }

                if self.files.is_none() {
                    self.files = Some(self.init_files());
                }
            }
        } else if let Some(dirs) = self.dirs.as_mut() {
            dirs.iter_mut().for_each(|dir| dir.update(path));
        }
    }

    /// Reloads the files in this directory, refreshing the internal list.
    fn reload_files(&mut self) {
        if let Some(_files) = &self.files {
            self.files = Some(self.init_files());
        } else if let Some(dirs) = self.dirs.as_mut() {
            dirs.iter_mut().for_each(|dir| dir.reload_files());
        }
    }

    /// Recursively builds the view of this directory and its children.
    ///
    /// Returns an [`Element`] representing the tree structure and total height.
    pub fn view(
        &self,
        clicked_path: &Option<PathBuf>,
        main_path: &Option<PathBuf>,
        selected_path: &Option<PathBuf>,
    ) -> (Element<'_, Message>, f32) {
        let mut col = column!(dir_button(
            self.name.clone(),
            Message::DirClick(self.path.clone()),
            self.is_open
        ));

        let mut height = HEIGHT;

        if self.is_open {
            let ch =
                column(
                    self.dirs
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|e| e.view(clicked_path, main_path, selected_path))
                        .chain(self.files.as_ref().unwrap().iter().map(|f| {
                            File::view(f, clicked_path, main_path, selected_path)
                        }))
                        .map(|(e, _h)| {
                            height += HEIGHT;
                            e
                        }),
                );

            col = col.push(row![
                space().width(INDENT),
                column![rule::vertical(1.0)]
                    .padding(padding::top(HEIGHT * 0.5 - 1.5).bottom(HEIGHT * 0.5 - 1.5))
                    .align_x(Alignment::Center)
                    .width(Length::Shrink)
                    .height(height),
                space().width(SPACING),
                ch
            ]);
        }

        (col.into(), height)
    }

    /// Initializes and returns the list of files in this directory.
    ///
    /// Files are sorted alphabetically (case-insensitive).
    fn init_files(&self) -> Vec<File> {
        match std::fs::read_dir(&self.path) {
            Ok(files) => {
                let mut files = files
                    .filter_map(Result::ok)
                    .filter(|file| file.file_type().is_ok_and(|t| t.is_file()))
                    .map(|file| {
                        let mut name = file.file_name();
                        name.make_ascii_lowercase();

                        (file, name)
                    })
                    .collect::<Vec<_>>();
                files.sort_unstable_by(|(_, aname), (_, bname)| aname.cmp(bname));
                files
                    .iter()
                    .map(|(entry, _)| File::new(&entry.path()))
                    .collect()
            }
            Err(e) => {
                println!("{} {:?}", e, self.path);
                [].into()
            }
        }
    }

    /// Initializes and returns the list of subdirectories.
    ///
    /// Directories are sorted alphabetically (case-insensitive).
    fn init_dirs(&self) -> Vec<Self> {
        let Ok(dirs) = std::fs::read_dir(&self.path) else {
            return [].into();
        };
        let mut dirs = dirs
            .filter_map(Result::ok)
            .filter(|file| file.file_type().is_ok_and(|t| t.is_dir()))
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .collect::<Vec<_>>();
        dirs.sort_unstable_by(|(_, aname), (_, bname)| aname.cmp(bname));
        dirs.iter()
            .map(|(entry, _)| Self::new(entry.path()))
            .collect()
    }
}

/// Represents a file node in the file system.
#[derive(Clone)]
pub struct File {
    /// The file path.
    path: PathBuf,
    /// The file display name.
    name: String,
}

impl File {
    /// Creates a new [`File`] from the given path.
    ///
    /// Automatically assigns an icon based on file extension.
    pub fn new(path: &PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap();
        Self {
            path: path.into(),
            name: name.into(),
        }
    }

    /// Builds and returns the view for the file node.
    /// Displays file name, icon, and optionally contextual actions (e.g. delete, mark main).
    ///
    /// Returns the element and its height.
    pub fn view(
        &self,
        clicked_path: &Option<PathBuf>,
        main_path: &Option<PathBuf>,
        selected_path: &Option<PathBuf>,
    ) -> (Element<'_, Message>, f32) {
        match clicked_path {
            Some(path) if self.path == *path => (
                container(
                    mouse_area(
                        row![
                            icon_button(
                                icon::flag(),
                                "Turn this file main",
                                Message::ChangeMainFile(self.path.clone())
                            ),
                            icon_button(
                                icon::trash(),
                                "Delete this file",
                                Message::DeleteFile(self.path.clone())
                            )
                        ]
                        .align_y(Alignment::Center)
                        .spacing(SPACING)
                        .width(Length::Fill),
                    )
                    .on_exit(Message::FileExit),
                )
                .into(),
                HEIGHT,
            ),
            _ => {
                let icon = match self.path.extension() {
                    Some(ext) => match ext.to_str() {
                        Some("png") | Some("jpg") | Some("jpeg") => icon::image_file(),
                        Some("pdf") => icon::pdf_file(),
                        Some("bib") | Some("toml") => icon::book(),
                        _ => icon::text_file(),
                    },
                    None => icon::text_file(),
                };
                let mut row =
                    row![icon, text(self.name.clone()).wrapping(Wrapping::None)]
                        .align_y(Alignment::Center)
                        .width(Length::Fill)
                        .spacing(SPACING);

                if let Some(path) = main_path {
                    if *path == self.path {
                        row = row.push(
                            text(" (main)").wrapping(Wrapping::None).style(main_style),
                        );
                    }
                }
                if let Some(path) = selected_path {
                    if self.path == *path {
                        (
                            container(
                                mouse_area(row)
                                    .on_press(Message::ChangeCurrentFile(
                                        self.path.clone(),
                                    ))
                                    .on_right_press(Message::FileClick(
                                        self.path.clone(),
                                    )),
                            )
                            .style(selected_file)
                            .into(),
                            HEIGHT,
                        )
                    } else {
                        (
                            container(
                                mouse_area(row)
                                    .on_press(Message::ChangeCurrentFile(
                                        self.path.clone(),
                                    ))
                                    .on_right_press(Message::FileClick(
                                        self.path.clone(),
                                    )),
                            )
                            .into(),
                            HEIGHT,
                        )
                    }
                } else {
                    (
                        container(
                            mouse_area(row)
                                .on_press(Message::ChangeCurrentFile(self.path.clone()))
                                .on_right_press(Message::FileClick(self.path.clone())),
                        )
                        .into(),
                        HEIGHT,
                    )
                }
            }
        }
    }
}

/// Creates a clickable button representing a directory.
///
/// Displays the name and toggle arrow based on open state.
fn dir_button<'a, Message: Clone + 'a>(
    name: String,
    on_press: Message,
    is_open: bool,
) -> Element<'a, Message> {
    mouse_area(
        row![
            space().width(10),
            if is_open {
                icon::open_dir()
            } else {
                icon::close_dir()
            }
            .width(18),
            text(name).wrapping(Wrapping::Word)
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(on_press)
    .into()
}

fn icon_button<'a, Message: Clone + 'a>(
    icon: Text<'a>,
    label: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    let action = button(
        icon.width(ICON_BUTTON_SIZE)
            .height(ICON_BUTTON_SIZE)
            .center(),
    )
    .width(ICON_BUTTON_SIZE)
    .height(ICON_BUTTON_SIZE)
    .clip(true)
    .style(toolbar_button)
    .on_press(on_press)
    .padding(4);

    tooltip(action, text(label), tooltip::Position::FollowCursor)
        .gap(20)
        .style(tooltip_box)
        .into()
}
