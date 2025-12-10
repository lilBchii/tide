use super::component::toolbar::{self, open_url, welcome_toolbar};
use crate::data::style::button::{files_button, toolbar_button};
use crate::file_manager::file::{
    get_recent_paths, get_templates_path, load_file_dialog, ProjectCache,
};
use crate::file_manager::import::TEMPLATE;
use crate::screen::component::modal;
use crate::screen::component::modal::ProjectModal;
use iced::advanced::text::Shaping;
use iced::widget::{button, row, space, stack, svg, text};
use iced::Length::Fill;
use iced::{
    widget::{column, container},
    Alignment, Element, Length, Task,
};
use std::path::PathBuf;
use std::sync::LazyLock;

const WELCOME_BUTTON_PADDING: f32 = 2.0;
const WELCOME_BUTTON_HEIGHT: f32 = 50.0;
const WELCOME_BUTTON_WIDTH: f32 = 200.0;
const FILE_BUTTON_HEIGHT: f32 = 30.0;
const H1_FONT_SIZE: f32 = 75.0;
const H2_FONT_SIZE: f32 = 20.0;
const SPACING: f32 = 35.0;
const TITLE_SPACING: f32 = 15.0;

pub static LOGO: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/thierry_colored.svg"))
});

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
    /// Handles interactions such as starting a new project or responding to modal
    /// actions.
    pub fn update(
        &mut self,
        message: Message,
    ) -> Task<Message> {
        match message {
            Message::ToolBar(message) => match message {
                toolbar::Message::StartFromTemplate => {
                    if let Some(template) = load_file_dialog(
                        &get_templates_path().unwrap_or(PathBuf::from("/")),
                        "Typst template",
                        &TEMPLATE,
                    ) {
                        self.project_modal.require_template(template);
                        return Task::done(Message::ToolBar(
                            toolbar::Message::NewProject,
                        ));
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
            Message::ProjectModal(message) => {
                self.project_modal.update(message).map(Message::ToolBar)
            }
        }
    }

    /// Renders the full welcome screen, including shortcuts, recent files, and modal
    /// if visible.
    pub fn view(&self) -> Element<Message> {
        let tool_bar = welcome_toolbar().map(Message::ToolBar);

        let title = row![
            svg(LOGO.to_owned()).width(120),
            text("Tide").shaping(Shaping::Advanced).size(H1_FONT_SIZE),
        ]
        .align_y(Alignment::Center);

        let getting_started = container(column![
            text("Getting started")
                .shaping(Shaping::Advanced)
                .size(H2_FONT_SIZE),
            space().height(TITLE_SPACING),
            welcome_button(
                "New Project",
                Message::ToolBar(toolbar::Message::NewProject),
            ),
            space().height(10),
            welcome_button(
                "Start from template",
                Message::ToolBar(toolbar::Message::StartFromTemplate),
            ),
            space().height(10),
            text("More options in “File” button of the tool bar.")
                .shaping(Shaping::Advanced)
        ]);

        let shortcuts = container(column![
            text("Shortcuts")
                .shaping(Shaping::Advanced)
                .size(H2_FONT_SIZE),
            space().height(TITLE_SPACING),
            row![
                space().width(SPACING),
                column![
                    text("Open File").shaping(Shaping::Advanced),
                    text("New Project").shaping(Shaping::Advanced),
                    text("Save File").shaping(Shaping::Advanced),
                    text("Preview").shaping(Shaping::Advanced),
                    text("Copy").shaping(Shaping::Advanced),
                    text("Paste").shaping(Shaping::Advanced),
                    text("Select All").shaping(Shaping::Advanced),
                ]
                .align_x(Alignment::End),
                space().width(SPACING),
                column![
                    text("Ctrl+O").shaping(Shaping::Advanced),
                    text("Ctrl+N").shaping(Shaping::Advanced),
                    text("Ctrl+S").shaping(Shaping::Advanced),
                    text("Ctrl+S").shaping(Shaping::Advanced),
                    text("Ctrl+C").shaping(Shaping::Advanced),
                    text("Ctrl+V").shaping(Shaping::Advanced),
                    text("Ctrl+A").shaping(Shaping::Advanced),
                ]
                .align_x(Alignment::Start)
            ]
        ]);

        let mut recent = column![
            text("Recent Projects")
                .shaping(Shaping::Advanced)
                .size(H2_FONT_SIZE),
            space().height(TITLE_SPACING),
        ];
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
            .height(5.0 * FILE_BUTTON_HEIGHT)
            .width(Length::Shrink);

        let r = row![
            getting_started,
            space().width(Fill),
            shortcuts,
            space().width(Fill)
        ];
        let c = container(
            column![title, r, recent_container]
                .spacing(SPACING)
                .align_x(Alignment::Start),
        )
        .padding(75);

        let screen = column![tool_bar, c];

        if self.project_modal.visible {
            return stack![screen, self.project_modal.view().map(Message::ProjectModal)]
                .into();
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
    button(text(label).shaping(Shaping::Advanced).width(150).center())
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
fn file_button<'a, Message: Clone + 'a>(
    label: String,
    on_press: Message,
) -> Element<'a, Message> {
    button(text(label).shaping(Shaping::Advanced).center())
        .on_press(on_press)
        .padding(WELCOME_BUTTON_PADDING)
        .width(Length::Shrink)
        .height(FILE_BUTTON_HEIGHT)
        .style(files_button)
        .into()
}
