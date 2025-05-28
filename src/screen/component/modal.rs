use crate::data::style::button::{cancel_button, toolbar_button, validate_button};
use crate::data::style::modal;
use crate::data::style::modal::modal_text_style;
use crate::screen::component::toolbar;
use crate::screen::editing;
use iced::widget::{button, center, column, horizontal_space, row, text, text_input};
use iced::widget::{container, opaque, vertical_space};
use iced::{Alignment, Color, Task};
use iced_core::text::Shaping;
use std::io;
use std::io::Result;
use std::path::PathBuf;

const MODAL_BUTTON_HEIGHT: u16 = 30;
const MODAL_BUTTON_WIDTH: u16 = 80;
#[derive(Debug, Clone)]
pub enum Message {
    /// Triggered when the user requests to create a file.
    FileCreate,
    /// Triggered when the file name input changes.
    FileNamed(String),
    /// Triggered when the user requests to create a project.
    ProjectCreate,
    /// Triggered when the project name input changes.
    ProjectName(String),
    /// Triggered when the project path input changes.
    ProjectPath(String),
    /// Triggered when the user opens a file dialog to choose a path.
    OpenFileDialog,
    /// Triggered when the modal is canceled.
    Cancel,
}

/// A modal dialog used to create a new file.
#[derive(Clone, Debug)]
pub struct FileModal {
    /// The name of the file being created.
    pub file_name: String,
    /// Warning text to show in case of invalid input.
    pub warning_text: String,
    /// The root path of the project in which the file modal is registered.
    pub root: PathBuf,
    /// The modal visibility state.
    pub visible: bool,
}

impl FileModal {
    /// Creates a new [`FileModal`] instance with empty fields.
    pub fn new(root: PathBuf) -> Self {
        Self {
            file_name: String::new(),
            warning_text: String::new(),
            root,
            visible: false,
        }
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Returns the Iced view for the modal.
    ///
    /// Includes a text input for the file name, a warning label,
    /// and action buttons for creating or canceling.
    pub fn view(&self) -> iced::Element<Message> {
        let modal_content = container(
            column![
                text("New File").size(20),
                vertical_space().height(10),
                row![text_input("Enter file name", &self.file_name)
                    .on_input(Message::FileNamed)],
                text(self.warning_text.clone())
                    .size(15)
                    .color(Color::from_rgb(1.0, 0.0, 0.0)),
                vertical_space().height(10),
                row![
                    button(text("Create").center())
                        .on_press(Message::FileCreate)
                        .height(MODAL_BUTTON_HEIGHT)
                        .width(MODAL_BUTTON_WIDTH)
                        .style(validate_button),
                    horizontal_space().width(10),
                    button(text("Cancel").center())
                        .on_press(Message::Cancel)
                        .height(MODAL_BUTTON_HEIGHT)
                        .width(MODAL_BUTTON_WIDTH)
                        .style(cancel_button),
                ],
            ]
            .align_x(Alignment::Center),
        )
        .width(300)
        .padding(20)
        .height(170)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(modal::modal_style);

        opaque(center(opaque(modal_content)))
    }

    /// Handles messages to update the internal state.
    pub fn update(
        &mut self,
        message: Message,
    ) -> Task<editing::Message> {
        match message {
            Message::FileCreate => {
                if !self.file_name.is_empty() {
                    let new_name = format!("{}.typ", self.file_name);
                    let path = self.root.join(new_name);
                    if !path.exists() {
                        self.hide();
                        return Task::done(editing::Message::CreateFile(path));
                    } else {
                        self.warning_text = String::from("File already exists!");
                    }
                } else {
                    self.warning_text = String::from("No file name");
                }
            }
            Message::FileNamed(name) => {
                self.file_name = name;
            }
            Message::Cancel => {
                self.file_name = String::new();
                self.warning_text = String::new();
                self.hide();
                println!("File name not set!");
            }
            _ => {}
        }
        Task::none()
    }
}

/// A modal dialog used to create a new project.
pub struct ProjectModal {
    /// The name of the new project.
    pub project_name: String,
    /// The path where the project should be created.
    pub project_path: String,
    /// Warning text to display for invalid inputs.
    pub warning_text: String,
    /// The modal visibility state.
    pub visible: bool,
    /// A template file that can be included in the project when it is created.
    pub template: Option<PathBuf>,
}

impl ProjectModal {
    /// Creates a new [ProjectModal] with empty input fields.
    pub fn new() -> Self {
        Self {
            project_name: String::new(),
            project_path: String::new(),
            warning_text: String::new(),
            visible: false,
            template: None,
        }
    }

    /// Makes the state of the modal invisible.
    fn hide(&mut self) {
        self.visible = false;
        self.template = None;
    }

    /// Makes the state of the modal visible.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Adds the path of a template file to be imported when creating a project.
    pub fn require_template(
        &mut self,
        template_path: PathBuf,
    ) {
        self.template = Some(template_path);
    }

    /// Returns the Iced view for the modal.
    ///
    /// Includes inputs for project name and path, a button to open a directory picker,
    /// and buttons to confirm or cancel the operation.
    pub fn view(&self) -> iced::Element<Message> {
        let modal_content = container(
            column![
                text("New Project").size(20),
                vertical_space().height(10),
                text_input("Enter project name", &self.project_name)
                    .on_input(Message::ProjectName),
                vertical_space().height(20),
                row![
                    text_input("Enter project absolute path", &self.project_path)
                        .on_input(Message::ProjectPath),
                    button(text("FD").center())
                        .on_press(Message::OpenFileDialog)
                        .style(toolbar_button)
                ],
                text(self.warning_text.clone())
                    .size(15)
                    .style(modal_text_style)
                    .shaping(Shaping::Advanced),
                vertical_space().height(10),
                row![
                    button(text("Create").center())
                        .on_press(Message::ProjectCreate)
                        .height(MODAL_BUTTON_HEIGHT)
                        .width(MODAL_BUTTON_WIDTH)
                        .style(validate_button),
                    horizontal_space().width(10),
                    button(text("Cancel").center())
                        .on_press(Message::Cancel)
                        .height(MODAL_BUTTON_HEIGHT)
                        .width(MODAL_BUTTON_WIDTH)
                        .style(cancel_button),
                ]
            ]
            .align_x(Alignment::Center),
        )
        .width(400)
        .padding(20)
        .height(215)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(modal::modal_style);

        opaque(center(opaque(modal_content)))
    }

    /// Handles messages and updates internal state accordingly.
    pub fn update(
        &mut self,
        message: Message,
    ) -> Task<toolbar::Message> {
        match message {
            Message::ProjectName(name) => {
                self.project_name = name;
            }

            Message::ProjectPath(path) => {
                self.project_path = path;
            }

            Message::Cancel => {
                self.project_path = String::new();
                self.project_name = String::new();
                self.warning_text = String::new();
                self.hide();
            }

            Message::OpenFileDialog => {
                let project_dir = rfd::FileDialog::new()
                    .pick_folder()
                    .and_then(|path| path.to_str().map(|s| s.to_string()));
                if let Some(project_dir) = project_dir {
                    self.project_path = project_dir;
                }
            }

            Message::ProjectCreate => {
                if !self.project_name.is_empty() {
                    if !self.project_path.is_empty() {
                        let new_project_path =
                            std::path::PathBuf::from(&self.project_path)
                                .join(&self.project_name);
                        if new_project_path.is_absolute() {
                            if new_project_path.exists() {
                                self.warning_text =
                                    String::from("Project already exists");
                            } else {
                                match self.create_project() {
                                    Ok(path) => {
                                        if path.exists() {
                                            self.hide();
                                            return Task::done(
                                                toolbar::Message::OpenProject(
                                                    Some(path),
                                                    None,
                                                ),
                                            );
                                        } else {
                                            self.warning_text =
                                                String::from("Project not found");
                                        }
                                    }
                                    Err(e) => {
                                        self.warning_text =
                                            format!("Error creating project: {}", e);
                                    }
                                }
                            }
                        } else {
                            self.warning_text = String::from("Path does not exist");
                        }
                    } else {
                        self.warning_text = String::from("No project path");
                    }
                } else {
                    self.warning_text = String::from("No project name");
                }
            }
            _ => {}
        }
        Task::none()
    }

    /// Attempts to create a new project directory with a default `main.typ` file inside.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the project path is empty, if the project path already exists
    /// or if directory/file creation fails.
    pub fn create_project(&self) -> Result<PathBuf> {
        if self.project_path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Project path is empty",
            ));
        }

        let path = std::path::Path::new(&self.project_path).join(&self.project_name);

        if path.exists() {
            println!("Project directory already exists: {}", path.display());
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Project path already exists",
            ));
        } else {
            std::fs::create_dir_all(&path)?;
            std::fs::File::create(path.join("main.typ"))?;
            if let Some(template) = &self.template {
                let template_path = path.join("template.typ");
                std::fs::File::create(&template_path)?;
                std::fs::copy(template, template_path)?;
            }
            println!("Project created at: {}", path.display());
        }

        Ok(path)
    }
}
