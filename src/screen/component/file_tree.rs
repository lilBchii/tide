use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::{
    data::style::{
        button::{drop_down_menu_button, simple_button},
        file_tree::{
            direntry_button, direntry_selected_button, drop_down_bg, main_style,
        },
        tooltip::tooltip_box,
    },
    icon,
    widgets::menu::menu,
};
use iced::{
    widget::{
        button, column, container, hover, row, scrollable, space, text, text::Wrapping,
        tooltip, Text,
    },
    Alignment, Element, Length,
};

//use iced_aw::{drop_down, DropDown};
use iced_palace::widget::ellipsized_text;

const INDENT: f32 = 10.0;
const SPACING: f32 = 5.0;
const PADDING: f32 = 4.0;

/// Represents messages triggered by user interactions with the file tree.
#[derive(Clone, Debug)]
pub enum Message {
    /// Set the given file as the main file for the project.
    ChangeMainFile(PathBuf),
    /// Toggle a directory's open/closed state on click.
    DirClick(PathBuf),
    /// Change the currently selected file to the one at the given path.
    ChangeCurrentFile(PathBuf),
    /// Delete the file at the specified path.
    DeleteFile(PathBuf),
    /// Delete the directory at the specified path.
    DeleteDirectory(PathBuf),

    Expand(PathBuf),
    Dismiss,

    AddFile(PathBuf),
    AddDirectory(PathBuf),
}

pub struct FileTree {
    directory: Directory,
    main_path: Option<PathBuf>,
    pub selected_path: Option<PathBuf>,
    // DropDown
    pub expanded_path: Option<PathBuf>,
}

impl FileTree {
    pub fn new(
        root: &Path,
        main_path: Option<PathBuf>,
        selected_path: Option<PathBuf>,
    ) -> Self {
        let directory = match tree_build(root) {
            Ok(dir) => dir,
            Err(_) => Directory::new(root),
        };
        Self {
            directory,
            main_path,
            selected_path,
            expanded_path: None,
        }
    }

    /// Toggles open/closed state of a directory node.
    pub fn fold(
        &mut self,
        path: &PathBuf,
    ) {
        self.directory.update(path);
    }

    /// Deletes a file from the file Tree
    pub fn delete_file(
        &mut self,
        file_path: &PathBuf,
    ) {
        self.directory.delete_file(file_path);
    }

    /// Deletes a directory from the file Tree
    pub fn delete_directory(
        &mut self,
        dir_path: &PathBuf,
    ) {
        self.directory.delete_directory(dir_path);
    }

    /// Changes the main Typst file path.
    ///
    /// If the path is already the main file, no change is made.
    pub fn change_main(
        &mut self,
        path: &Path,
    ) {
        self.main_path = Some(path.to_path_buf());
    }

    /// Changes the currently selected file.
    pub fn change_selected(
        &mut self,
        path: &Path,
    ) {
        self.selected_path = Some(path.to_path_buf());
    }

    /// Adds a new empty directory to the tree
    pub fn add_new_directory(
        &mut self,
        path: &PathBuf,
    ) {
        self.directory.add_new_directory(path);
    }

    /// Adds a new empty directory to the tree
    pub fn add_new_file(
        &mut self,
        path: &PathBuf,
    ) {
        self.directory.add_new_file(path);
    }
}

struct Directory {
    path: PathBuf,
    name: String,

    directories: Vec<Directory>,
    files: Vec<File>,
    is_open: bool,
}

impl Directory {
    fn new(root: &Path) -> Self {
        Self {
            path: root.to_path_buf(),
            name: root.file_name().unwrap().to_string_lossy().to_string(),
            directories: vec![],
            files: vec![],
            is_open: false,
        }
    }

    fn update(
        &mut self,
        path: &PathBuf,
    ) {
        if &*self.path == path {
            self.is_open ^= true;
        } else if !self.directories.is_empty() {
            self.directories.iter_mut().for_each(|dir| dir.update(path));
        }
    }

    fn delete_file(
        &mut self,
        file_path: &PathBuf,
    ) {
        if let Some(dir_path) = file_path.parent() {
            if dir_path == self.path {
                self.files.retain(|file| &file.path != file_path);
            } else {
                self.directories
                    .iter_mut()
                    .for_each(|dir| dir.delete_file(file_path));
            }
        }
    }

    fn delete_directory(
        &mut self,
        dir_path: &PathBuf,
    ) {
        if let Some(parent_path) = dir_path.parent() {
            if parent_path == self.path {
                self.directories.retain(|dir| &dir.path != dir_path);
            } else {
                self.directories
                    .iter_mut()
                    .for_each(|dir| dir.delete_directory(dir_path));
            }
        }
    }

    fn add_new_file(
        &mut self,
        file_path: &PathBuf,
    ) {
        if let Some(dir_path) = file_path.parent() {
            if dir_path == self.path {
                self.files.push(File {
                    name: file_path.file_name().unwrap().to_string_lossy().to_string(), // TODO: remove unwrap
                    path: file_path.to_path_buf(),
                });
                self.files.sort_by(|a, b| a.name.cmp(&b.name));
            } else {
                self.directories
                    .iter_mut()
                    .for_each(|dir| dir.add_new_file(file_path));
            }
        }
    }

    fn add_new_directory(
        &mut self,
        dir_path: &PathBuf,
    ) {
        if let Some(parent_path) = dir_path.parent() {
            if parent_path == self.path {
                self.directories.push(Directory::new(dir_path));
                self.directories.sort_by(|a, b| a.name.cmp(&b.name));
            } else {
                self.directories
                    .iter_mut()
                    .for_each(|dir| dir.add_new_directory(dir_path));
            }
        }
    }
}

struct File {
    name: String,
    path: PathBuf,
}

/// Recursively builds the tree from root path, which has to be a directory/folder
fn tree_build(root: &Path) -> std::io::Result<Directory> {
    let mut directory = Directory::new(root);

    for entry in read_dir(root)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            directory.files.push(File {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path(),
            })
        } else if metadata.is_dir() {
            directory.directories.push(tree_build(&entry.path())?);
        }
    }

    directory.directories.sort_by(|a, b| a.name.cmp(&b.name));
    directory.files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(directory)
}

fn view_file<'a>(
    file: &'a File,
    main_path: &'a Option<PathBuf>,
    selected_path: &'a Option<PathBuf>,
) -> Element<'a, Message> {
    let icon = match file.path.extension() {
        Some(ext) => match ext.to_str() {
            Some("png") | Some("jpg") | Some("jpeg") => icon::image_file(),
            Some("pdf") => icon::pdf_file(),
            Some("bib") | Some("toml") => icon::book(),
            _ => icon::text_file(),
        },
        None => icon::text_file(),
    };

    let content = row![
        icon.center(),
        ellipsized_text(file.name.clone())
            .wrapping(Wrapping::None)
            .center()
    ]
    .push(main_path.as_ref().and_then(|path| {
        (file.path == *path).then(|| {
            text(" (main)")
                .wrapping(Wrapping::None)
                .style(main_style)
                .center()
        })
    }))
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .spacing(SPACING);

    // TODO: make text ellipsize before hover buttons to avoid unreadability
    let hovered_content = row![
        space().width(Length::Fill),
        icon_button(
            icon::trash(),
            "Delete",
            Message::DeleteFile(file.path.clone())
        ),
        icon_button(
            icon::flag(),
            "Mark as main",
            Message::ChangeMainFile(file.path.clone())
        )
    ]
    .align_y(Alignment::Center);

    let mut file_button = button(content)
        .on_press(Message::ChangeCurrentFile(file.path.clone()))
        .style(direntry_button);

    if let Some(selected_path) = selected_path {
        if selected_path == &file.path {
            file_button = file_button.style(direntry_selected_button);
        }
    }

    hover(file_button, hovered_content)
}

fn view_directory<'a>(
    directory: &'a Directory,
    main_path: &'a Option<PathBuf>,
    selected_path: &'a Option<PathBuf>,
    extanded_path: &'a Option<PathBuf>,
) -> Element<'a, Message> {
    let hovered_parent = row![
        space().width(Length::Fill),
        icon_button(
            icon::trash(),
            "Delete",
            Message::DeleteDirectory(directory.path.to_path_buf()),
        ),
        menu(
            icon_button(
                icon::plus(),
                "New",
                Message::Expand(directory.path.to_path_buf())
            ),
            container(
                column![
                    button("Add New File")
                        .width(Length::Fill)
                        .on_press(Message::AddFile(directory.path.to_path_buf()))
                        .style(drop_down_menu_button),
                    button("Add Directory")
                        .width(Length::Fill)
                        .on_press(Message::AddDirectory(directory.path.to_path_buf()))
                        .style(drop_down_menu_button),
                ]
                .width(120.0)
                .align_x(Alignment::Start)
            )
            .style(drop_down_bg),
            extanded_path
                .as_ref()
                .is_some_and(|extanded_path| extanded_path == &directory.path)
        )
        .on_dismiss(Message::Dismiss)
    ]
    .align_y(Alignment::Center);

    let parent = row![]
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .spacing(SPACING);

    if directory.is_open {
        let parent = parent
            .push(icon::open_dir().center())
            .push(text(directory.name.clone()).center());

        let parent = button(parent)
            .on_press(Message::DirClick(directory.path.clone()))
            .style(direntry_button);

        let parent = hover(parent, hovered_parent);

        let inside = column(
            directory
                .directories
                .iter()
                .map(|sub_dir| {
                    view_directory(sub_dir, main_path, selected_path, extanded_path)
                })
                .chain(
                    directory
                        .files
                        .iter()
                        .map(|file| view_file(file, main_path, selected_path)),
                ),
        );

        let pretty = row![space().width(INDENT), inside]
            .height(Length::Shrink)
            .spacing(SPACING);

        container(column![parent, pretty]).into()
    } else {
        let parent = button(
            parent
                .push(icon::close_dir().center())
                .push(text(directory.name.clone()).center()),
        )
        .on_press(Message::DirClick(directory.path.clone()))
        .style(direntry_button);

        hover(parent, hovered_parent)
    }
}

pub fn view_file_tree<'a>(tree: &'a FileTree) -> Element<'a, Message> {
    container(
        scrollable(view_directory(
            &tree.directory,
            &tree.main_path,
            &tree.selected_path,
            &tree.expanded_path,
        ))
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .scroller_width(PADDING)
                .width(PADDING)
                .spacing(SPACING),
        )),
    )
    .height(Length::Fill)
    .padding(PADDING)
    .into()
}

/// Creates an icon-based button with a tooltip, styled for the toolbar.
///
/// `label` is the tooltip text displayed on hover,
/// and `on_press` the message to emit when the button is pressed.
fn icon_button<'a, Message: Clone + 'a>(
    icon: Text<'a>,
    label: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    let action = button(icon.center())
        .width(Length::Shrink)
        .height(Length::Shrink)
        .clip(true)
        .style(simple_button);

    tooltip(
        action.on_press(on_press),
        text(label),
        tooltip::Position::Bottom,
    )
    .style(tooltip_box)
    .into()
}
