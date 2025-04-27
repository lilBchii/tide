use super::component::toolbar::{self, open_url, welcome_toolbar};
use crate::data::style::button::{files_button, toolbar_button};
use crate::file_manager::file::{
    get_recent_paths, get_templates_path, load_file_dialog, ProjectCache,
};
use crate::file_manager::import::TEMPLATE;
use crate::screen::component::modal;
use crate::screen::component::modal::ProjectModal;
use iced::advanced::text::Shaping;
use iced::widget::{horizontal_space, row, stack, svg, text, vertical_space, Button, Text};
use iced::{
    widget::{column, container},
    Alignment, Element, Length, Task,
};
use std::path::PathBuf;
use std::sync::LazyLock;

const WELCOME_BUTTON_PADDING: u16 = 2;
const WELCOME_BUTTON_HEIGHT: u16 = 50;
const WELCOME_BUTTON_WIDTH: u16 = 200;
const FILE_BUTTON_HEIGHT: u16 = 30;

pub static LOGO: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../../assets/icons/logo.svg")));

/// Represents the main _Welcome page_ shown on Tide startup.
///
/// Displays shortcuts, recent projects, and project creation options.
pub struct Welcome {
    /// List of recently opened projects.
    recent_files: Vec<ProjectCache>,
    /// The modal used for creating a new project.
    project_modal: ProjectModal,
}

/// Messages handled by the _Welcome_ view.
#[derive(Debug, Clone)]
pub enum Message {
    /// Forwarded message from the toolbar.
    ToolBar(toolbar::Message),
    /// Forwarded message from the project creation modal.
    ProjectModal(modal::Message),
}

impl Welcome {
    /// Creates a new [`Welcome`] view with cached recent projects.
    pub fn new() -> Self {
        Self {
            recent_files: get_recent_paths(),
            project_modal: ProjectModal::new(),
        }
    }

    /// Updates the welcome view based on incoming [`Message`]s.
    ///
    /// Handles interactions such as starting a new project or responding to modal actions.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToolBar(message) => match message {
                toolbar::Message::StartFromTemplate => {
                    if let Some(template) = load_file_dialog(
                        &get_templates_path().unwrap_or(PathBuf::from("/")),
                        "Typst template",
                        &TEMPLATE,
                    ) {
                        self.project_modal.require_template(template);
                        return Task::done(Message::ToolBar(toolbar::Message::NewProject));
                    }
                    Task::none()
                }
                toolbar::Message::NewProject => {
                    self.project_modal.show();
                    Task::none()
                }
                toolbar::Message::Help => {
                    open_url("https://typst.app/docs/");
                    Task::none()
                }
                toolbar::Message::Universe => {
                    open_url("https://typst.app/universe/");
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::ProjectModal(message) => self
                .project_modal
                .update(message)
                .map(|e| Message::ToolBar(e)),
        }
    }

    /// Renders the full welcome screen, including shortcuts, recent files, and modal if visible.
    pub fn view(&self) -> Element<Message> {
        let tool_bar = welcome_toolbar().map(Message::ToolBar);
        let shortcut_text = text("Shortcuts").shaping(Shaping::Advanced).size(20);
        let shortcuts = container(column![
            shortcut_text,
            row![
                horizontal_space().width(20),
                column![
                    Text::new("Open File").shaping(Shaping::Advanced),
                    Text::new("New Project").shaping(Shaping::Advanced),
                    Text::new("Save File").shaping(Shaping::Advanced),
                    Text::new("Preview").shaping(Shaping::Advanced),
                    Text::new("Copy").shaping(Shaping::Advanced),
                    Text::new("Paste").shaping(Shaping::Advanced),
                    Text::new("Select All").shaping(Shaping::Advanced),
                ]
                .align_x(Alignment::End),
                horizontal_space().width(20),
                column![
                    Text::new("Ctrl+O").shaping(Shaping::Advanced),
                    Text::new("Ctrl+N").shaping(Shaping::Advanced),
                    Text::new("Ctrl+S").shaping(Shaping::Advanced),
                    Text::new("Ctrl+S").shaping(Shaping::Advanced),
                    Text::new("Ctrl+C").shaping(Shaping::Advanced),
                    Text::new("Ctrl+V").shaping(Shaping::Advanced),
                    Text::new("Ctrl+A").shaping(Shaping::Advanced),
                ]
                .align_x(Alignment::Start)
            ]
        ]);
        let new_button = welcome_button(
            "New Project",
            Message::ToolBar(toolbar::Message::NewProject),
        );
        let start_button = welcome_button(
            "Start from template",
            Message::ToolBar(toolbar::Message::StartFromTemplate),
        );
        let title = text("Tide").shaping(Shaping::Advanced).size(75);
        let recent_text = text("Recent Projects").shaping(Shaping::Advanced).size(20);
        let mut recent = column![];
        for project in self.recent_files.iter() {
            let dir = project.root_path.to_owned();
            let main = project.main.to_owned();
            if let (Some(name), Some(path_str)) =
                (dir.file_name().and_then(|s| s.to_str()), dir.to_str())
            {
                let label = format!("{name}    {path_str}");
                let button = file_button(
                    label,
                    Message::ToolBar(toolbar::Message::OpenProject(Some(dir), main)),
                );
                recent = recent.push(button);
            }
        }

        let recent_container = container(recent)
            .height(5 * FILE_BUTTON_HEIGHT)
            .width(Length::Shrink);
        let getting_started = column![
            text("Getting started").shaping(Shaping::Advanced).size(20),
            vertical_space().height(10),
            new_button,
            vertical_space().height(20),
            start_button,
            vertical_space().height(10),
            text("More options in “File” button of the tool bar.").shaping(Shaping::Advanced)
        ];
        let r = row![
            getting_started,
            horizontal_space(),
            shortcuts,
            horizontal_space()
        ];
        let c = column![
            row![svg(LOGO.to_owned()).width(100), title],
            r,
            vertical_space().height(30),
            recent_text,
            recent_container
        ];

        let screen = column![
            tool_bar,
            vertical_space().height(50),
            row![horizontal_space().width(50), c]
        ];

        if self.project_modal.visible {
            return stack![screen, self.project_modal.view().map(Message::ProjectModal)].into();
        }

        screen.into()
    }
}

/// Creates a (big) button for the welcome screen with standard style.
///
/// Used for actions like "New Project" or "Start from template".
fn welcome_button<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    Button::new(
        Text::new(label)
            .shaping(Shaping::Advanced)
            .width(150)
            .center(),
    )
    .on_press(on_press)
    .padding(WELCOME_BUTTON_PADDING)
    .width(WELCOME_BUTTON_WIDTH)
    .height(WELCOME_BUTTON_HEIGHT)
    .style(toolbar_button)
    .into()
}

/// Creates a clickable button representing a recent file or project.
///
/// Displays the project name and path.
fn file_button<'a, Message: Clone + 'a>(label: String, on_press: Message) -> Element<'a, Message> {
    Button::new(Text::new(label).shaping(Shaping::Advanced).center())
        .on_press(on_press)
        .padding(WELCOME_BUTTON_PADDING)
        .width(Length::Shrink)
        .height(FILE_BUTTON_HEIGHT)
        .style(files_button)
        .into()
}
