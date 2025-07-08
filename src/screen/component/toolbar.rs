use crate::data::style::button::{drop_down_menu_button, toolbar_button};
use crate::data::style::tooltip::tooltip_box;
use crate::file_manager::export::errors::ExportError;
use crate::file_manager::export::ExportType;
use crate::file_manager::import::UploadType;
use iced::widget::tooltip;
use iced::widget::{button, horizontal_space, row, svg, text, Button};
use iced::{alignment, Element, Length, Theme};
use iced_aw::menu::{Item, Menu};
use iced_aw::{menu_bar, menu_items};
use open::that;
use std::sync::LazyLock;
use std::{io::ErrorKind, path::PathBuf};
use typst::syntax::VirtualPath;

const ICON_BUTTON_SIZE: u16 = 24;
const ICON_BUTTON_PADDING: u16 = 4;
const TOOLBAR_PADDING: u16 = 2;
const TOOLBAR_SPACING: u16 = 6;
const PREVIEW_BUTTON_SIZE: u16 = 60;
const MAIN_BUTTON_SIZE: u16 = 40;
const MENU_BUTTON_SIZE: u16 = 50;
const TEXT_BUTTON_PADDING: u16 = 2;

static TYPST_UNIVERSE_ICON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!(
        "../../../assets/icons/typst_universe_icon.svg"
    ))
});
static TYPST_DOCS_ICON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../../assets/icons/typst_help_icon.svg"))
});
static TYPST_QUICK_EXPORT_ICON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!(
        "../../../assets/icons/typst_quick_export_icon.svg"
    ))
});

/// Messages used in the context of toolbar interactions.
#[derive(Debug, Clone)]
pub enum Message {
    /// Forces the preview to regenerate.
    ForcePreview,
    /// Toggles the visibility or style of the preview.
    TogglePreview,
    /// Triggers file saving. The boolean indicates whether it's a "Save As".
    SaveFile(bool),
    /// Indicates the result of a file save operation.
    FileSaved(Result<PathBuf, ErrorKind>),
    /// Creates a new file.
    NewFile,
    /// Creates a new project.
    NewProject,
    /// Initiates creation from a predefined template.
    StartFromTemplate,
    /// Indicates the result of starting from a template.
    StartedFromTemplate(Result<PathBuf, ErrorKind>),
    /// Adds a local template to the project.
    AddTemplate,
    /// Triggers theme switching.
    ChangeTheme,
    /// A specific theme was selected.
    ThemeSelected(Theme),
    /// Opens a file from the system.
    OpenFile,
    /// Opens a project from the system. Accepts optional path and main file path (if it's loaded from cache).
    OpenProject(Option<PathBuf>, Option<PathBuf>),
    /// Initiates an export operation.
    Export(ExportType),
    /// Indicates the result of an export operation.
    ProjectExported(Result<PathBuf, ExportError>),
    /// Opens the Typst documentation online.
    Help,
    /// Opens the Typst Universe online.
    Universe,
    /// Opens or toggles a drop-down menu.
    DropDownMenu,
    /// Indicates the result of an imported file.
    FileImported(Result<PathBuf, ErrorKind>),
    /// Uploads a file of a given type, optionally providing a path.
    Upload(UploadType, Option<PathBuf>),
}

/// Returns the editing toolbar element for the application.
///
/// This toolbar includes drop-down menus for file management, theme switching,
/// and export actions, as well as quick-access buttons for help, preview, and Typst Universe.
///
/// The `main_path` is an optional reference to the currently selected main file (virtual path).
pub fn editing_toolbar<'a>(main_path: Option<&VirtualPath>) -> Element<'a, Message> {
    let menu_tpl_1 = |items| {
        Menu::new(items)
            .max_width(300.0)
            .offset(TOOLBAR_SPACING as f32)
            .spacing(TOOLBAR_SPACING * 2)
    };

    #[rustfmt::skip]
    let menu_bar = menu_bar!(
        (text_button("File", Message::DropDownMenu, MENU_BUTTON_SIZE), {
            menu_tpl_1(menu_items!(
                (menu_button("New File",Message::NewFile))
                (menu_button("Open File", Message::OpenFile))
                (menu_button("New Project", Message::NewProject))
                (menu_button("Open Project", Message::OpenProject(None, None)))
                (menu_button("Upload File", Message::Upload(UploadType::All, None)))
                (menu_button("Save", Message::SaveFile(true)))
                (menu_button("Save as", Message::SaveFile(true)))
                (menu_button("Export as", Message::DropDownMenu), menu_tpl_1(menu_items!(
                    (menu_button("PDF", Message::Export(ExportType::PDF)))
                    (menu_button("SVG", Message::Export(ExportType::SVG)))
                    (menu_button("Template", Message::Export(ExportType::Template)))
                )))
                (menu_button("Add Local Template", Message::AddTemplate))
                (menu_button("Start from Template", Message::StartFromTemplate))
            )).width(240.0)
        })
        (text_button("View", Message::DropDownMenu, MENU_BUTTON_SIZE), menu_tpl_1(menu_items!(
            (menu_button("Theme", Message::ChangeTheme))
            (menu_button("Invert", Message::TogglePreview))
        )).width(240.0))
    );

    let universe_button = icon_button(
        TYPST_UNIVERSE_ICON.clone(),
        "Typst Universe",
        Message::Universe,
    );

    let help_button = icon_button(TYPST_DOCS_ICON.clone(), "Help", Message::Help);

    let export_button = icon_button(
        TYPST_QUICK_EXPORT_ICON.clone(),
        "Quick Export",
        Message::Export(ExportType::PDF),
    );

    let main_file_path = match main_path {
        Some(path) => text(format!("{:?}", path.clone())),
        None => text("No main file..."),
    };
    let preview_button =
        text_button("Preview", Message::ForcePreview, PREVIEW_BUTTON_SIZE);

    let r = row![
        horizontal_space().width(TOOLBAR_SPACING),
        menu_bar,
        horizontal_space(),
        preview_button,
        main_file_path,
        horizontal_space(),
        universe_button,
        help_button,
        export_button,
        horizontal_space().width(TOOLBAR_SPACING),
    ]
    .align_y(alignment::Alignment::Center)
    .height(ICON_BUTTON_SIZE + TOOLBAR_PADDING * 2)
    .spacing(TOOLBAR_SPACING);
    r.into()
}

/// Returns the toolbar displayed on the welcome screen (before a project is opened).
///
/// This version of the toolbar includes fewer options and emphasizes
/// project creation or file opening actions.
pub fn welcome_toolbar<'a>() -> Element<'a, Message> {
    let menu_tpl_1 = |items| {
        Menu::new(items)
            .max_width(300.0)
            .offset(TOOLBAR_SPACING as f32)
            .spacing(TOOLBAR_SPACING * 2)
    };

    #[rustfmt::skip]
    let menu_bar = menu_bar!(
        (text_button("File", Message::DropDownMenu, MENU_BUTTON_SIZE), {
            menu_tpl_1(menu_items!(
                (menu_button("New File",Message::NewFile))
                (menu_button("Open File", Message::OpenFile))
                (menu_button("New Project", Message::NewProject))
                (menu_button("Open Project", Message::OpenProject(None, None)))
                (menu_button("Start from Template", Message::StartFromTemplate))
            )).width(240.0)
        })
        (text_button("View", Message::DropDownMenu, MENU_BUTTON_SIZE), menu_tpl_1(menu_items!(
            (menu_button("Theme", Message::ChangeTheme))
        )).width(240.0))
    );

    let universe_button = icon_button(
        TYPST_UNIVERSE_ICON.clone(),
        "Typst Universe",
        Message::Universe,
    );

    let help_button = icon_button(TYPST_DOCS_ICON.clone(), "Help", Message::Help);

    let r = row![
        horizontal_space().width(TOOLBAR_SPACING),
        menu_bar,
        horizontal_space(),
        horizontal_space(),
        universe_button,
        help_button,
        horizontal_space().width(TOOLBAR_SPACING),
    ]
    .align_y(alignment::Alignment::Center)
    .height(ICON_BUTTON_SIZE + TOOLBAR_PADDING * 2)
    .spacing(TOOLBAR_SPACING);

    r.into()
}

/// Creates a styled button used inside drop-down menus.
///
/// `label` is the text label of the menu button and `message` is the message to emit on press.
fn menu_button(
    label: &str,
    message: Message,
) -> Button<Message> {
    Button::new(text(label))
        .on_press(message)
        .style(drop_down_menu_button)
        .width(Length::Fill)
}

/// Creates an icon-based button with a tooltip, styled for the toolbar.
///
/// `path` is the path to the image asset used as an icon, `label` the tooltip text displayed on hover,
/// and `on_press` the message to emit when the button is pressed.
fn icon_button<'a, Message: Clone + 'a>(
    bytes: svg::Handle,
    label: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    let action = button(svg(bytes).width(24).height(24))
        .width(ICON_BUTTON_SIZE)
        .height(ICON_BUTTON_SIZE)
        .clip(true)
        .style(toolbar_button)
        .padding(ICON_BUTTON_PADDING);

    tooltip(
        action.on_press(on_press),
        text(label),
        tooltip::Position::FollowCursor,
    )
    .gap(20)
    .style(tooltip_box)
    .into()
}

/// Creates a text-based button styled for use in toolbars or menu bars.
///
/// `label` is the button's visible text, `on_press` is the message to emit on press and
/// `width` is the width of the button.
fn text_button<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: Message,
    width: u16,
) -> Element<'a, Message> {
    Button::new(text(label).width(20).center())
        .on_press(on_press)
        .padding(TEXT_BUTTON_PADDING)
        .width(width)
        .height(ICON_BUTTON_SIZE)
        .style(toolbar_button)
        .into()
}

/// Attempts to open a given URL or file `path` in the system's default handler.
pub fn open_url(path: &str) {
    match that(path) {
        Ok(()) => println!("link '{}' opened successfully.", path),
        Err(err) => println!("an error occurred when opening '{}': {}", path, err),
    }
}
