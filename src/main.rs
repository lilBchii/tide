use std::borrow::Cow;

use font::{
    APP_BOLD_BYTES, APP_FONT_FAMILY_NAME, APP_ITALIC_BYTES, APP_REG_BYTES,
    APP_SEMI_BOLD_BYTES, EDITOR_BOLD_BYTES, EDITOR_ITALIC_BYTES, EDITOR_REG_BYTES,
    EDITOR_SEMI_BOLD_BYTES,
};
use iced::Font;
use iced::{widget::container, Element, Settings, Task, Theme};

mod data;
mod editor;
mod file_manager;
mod font;
mod screen;
mod widgets;
mod world;

use crate::file_manager::file::{get_config_path, load_repo_dialog};
use crate::screen::component::toolbar;
use crate::screen::welcome::Welcome;
use data::config::appearance::{Config, GeneralConfig};
use screen::{
    editing::{self, Editing},
    welcome::{self},
    Screen,
};

/// Entry point of the Tide application.
///
/// Loads configuration from a TOML file and launches the application using `iced::application`.
fn main() -> iced::Result {
    let config = Config::load(get_config_path());
    let settings = settings(&config.general);

    iced::application(move || Tide::new(config.clone()), Tide::update, Tide::view)
        .settings(settings)
        .theme(Tide::theme)
        //.font(config.retrieve_font()) //todo: .default_font(Font::with_name())
        .resizable(true)
        .transparent(true)
        .scale_factor(Tide::scale_factor)
        .run()
}

/// The main application structure for Tide.
///
/// Manages the current screen state, theming, configuration, and window scale.
struct Tide {
    /// The currently active screen (either the welcome screen or the editor).
    screen: Screen,
    /// The current theme used across the application UI.
    theme: Theme,
    /// The factor used to scale UI elements based on the display.
    window_scale_factor: f64,
    /// The configuration settings loaded from a TOML file.
    config: Config,
}

impl Tide {
    /// Constructs a new [`Tide`] instance.
    ///
    /// Initializes the welcome screen, theme, scale factor, and stores the config.
    fn new(config: Config) -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Welcome(Welcome::new()),
                theme: Theme::from(&config.colors),
                window_scale_factor: config.general.window_scale_factor,
                config,
            },
            Task::none(),
        )
    }

    /// Retrieves the current window scale factor.
    //iced uses winit to create the window, but to this day, devs haven't found a "magic" way
    //to find the scale factor on all platforms (see https://docs.rs/winit-gtk/latest/winit/dpi/index.html#how-is-the-scale-factor-calculated)
    //especially for X11 (https://github.com/iced-rs/iced/issues/2657)
    //as this is still an opened issue,
    //the best choice is to leave scale factor at 1.0 by default and let the user change it in
    //the application configuration if winit is unable to find it
    fn scale_factor(&self) -> f64 {
        self.window_scale_factor
    }

    /// Handles application updates in response to messages from the UI.
    ///
    /// This includes message handling for both the editor and the welcome screen.
    /// Transitions from welcome to editor mode based on user interaction.
    fn update(
        &mut self,
        message: Message,
    ) -> Task<Message> {
        match message {
            Message::Editor(message) => {
                let Screen::Editing(editing) = &mut self.screen else {
                    return Task::none();
                };
                let task = editing.update(message);
                task.map(Message::Editor)
            }
            Message::Welcome(message) => {
                let Screen::Welcome(welcome) = &mut self.screen else {
                    return Task::none();
                };
                match message {
                    welcome::Message::ToolBar(message) => match message {
                        toolbar::Message::DropDownMenu => Task::none(),
                        toolbar::Message::OpenProject(path_option, main) => {
                            match path_option {
                                Some(import_path) => {
                                    self.screen = Screen::Editing(Editing::new(
                                        self.config.editor.clone(),
                                        import_path.to_path_buf(),
                                    ));
                                    Task::done(Message::Editor(
                                        editing::Message::ToolBar(
                                            toolbar::Message::OpenProject(
                                                Some(import_path),
                                                main,
                                            ),
                                        ),
                                    ))
                                }
                                None => {
                                    let dialog_path = load_repo_dialog();
                                    match dialog_path {
                                        Some(import_path) => {
                                            self.screen = Screen::Editing(Editing::new(
                                                self.config.editor.clone(),
                                                import_path.to_path_buf(),
                                            ));
                                            Task::done(Message::Editor(
                                                editing::Message::ToolBar(
                                                    toolbar::Message::OpenProject(
                                                        Some(import_path),
                                                        main,
                                                    ),
                                                ),
                                            ))
                                        }
                                        None => Task::none(),
                                    }
                                }
                            }
                        }
                        _ => welcome
                            .update(welcome::Message::ToolBar(message))
                            .map(Message::Welcome),
                    },
                    welcome::Message::ProjectModal(message) => welcome
                        .update(welcome::Message::ProjectModal(message))
                        .map(Message::Welcome),
                }
            }
        }
    }

    /// Renders the current UI based on the active screen.
    ///
    /// Delegates rendering to either the editor or the welcome view.
    fn view(&self) -> Element<Message> {
        let screen = match &self.screen {
            Screen::Editing(editing) => editing.view().map(Message::Editor),
            Screen::Welcome(welcome) => welcome.view().map(Message::Welcome),
        };
        container(screen).into()
    }

    /// Returns the current UI theme.
    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

/// Represents messages that trigger updates in the Tide application.
///
/// Each variant corresponds to a high-level screen state in the UI.
#[derive(Debug, Clone)]
enum Message {
    Editor(editing::Message),
    Welcome(welcome::Message),
}

fn settings(config: &GeneralConfig) -> Settings {
    Settings {
        fonts: vec![
            Cow::Borrowed(APP_REG_BYTES),
            Cow::Borrowed(APP_ITALIC_BYTES),
            Cow::Borrowed(APP_SEMI_BOLD_BYTES),
            Cow::Borrowed(APP_BOLD_BYTES),
            Cow::Borrowed(EDITOR_REG_BYTES),
            Cow::Borrowed(EDITOR_ITALIC_BYTES),
            Cow::Borrowed(EDITOR_SEMI_BOLD_BYTES),
            Cow::Borrowed(EDITOR_BOLD_BYTES),
        ],
        default_font: Font::with_name(APP_FONT_FAMILY_NAME),
        default_text_size: config.font_size.into(),
        ..Default::default()
    }
}
