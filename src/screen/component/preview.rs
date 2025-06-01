use iced::widget::scrollable::{AbsoluteOffset, Direction, Scrollbar, Viewport};
use iced::{
    advanced::image::{Bytes, FilterMethod, Handle},
    widget::{button, column, container, image, row, scrollable, text},
    Alignment, Element, Length, Task, Theme,
};

use typst::layout::{Page, PagedDocument};
use typst_render::render;

const SPACING_BETWEEN_PAGES: u16 = 15;
const SCROLL_RELOAD_DELAY: f32 = 50.0;
const DEFAULT_VIEWPORT_H: f32 = 1500.0;

#[derive(Debug, Clone)]
pub struct PagePreview {
    bytes: Option<Bytes>,
    height: f32,
    width: f32,
}

impl PagePreview {
    fn empty(page: &Page) -> Self {
        Self {
            bytes: None,
            height: page.frame.height().to_pt() as f32,
            width: page.frame.width().to_pt() as f32,
        }
    }

    fn fetch(page: &Page, zoom: f32) -> Self {
        let encode_result = render(page, 2.0 * zoom).encode_png(); //should take into account the window scale factor (see appearance.rs)

        Self {
            bytes: encode_result.ok().map(Bytes::from_owner),
            height: page.frame.height().to_pt() as f32,
            width: page.frame.width().to_pt() as f32,
        }
    }

    fn view(&self, zoom: f32) -> Element<Message> {
        if let Some(bytes) = &self.bytes {
            image(Handle::from_bytes(bytes.clone()))
                .width(self.width * zoom)
                .height(self.height * zoom)
                .filter_method(FilterMethod::Linear)
                .into()
        } else {
            container("")
                .style(|theme: &Theme| {
                    container::background(
                        theme
                            .extended_palette()
                            .background
                            .weak
                            .color
                            .scale_alpha(0.5),
                    )
                })
                .height(self.height * zoom)
                .width(self.width * zoom)
                .into()
        }
    }
}

/// Represents the current state of the document preview, including rendering handles and display mode.
pub struct Preview {
    zoom: f32,
    viewport_h: f32,
    offset: AbsoluteOffset,
    pages_to_render: (usize, usize),
    pages: Option<Vec<PagePreview>>,
}

#[derive(Clone, Debug)]
pub enum Message {
    IncrementZoom,
    DecrementZoom,
    Scrolled(Viewport),
    ReloadPages,
    PagesReloaded(Vec<PagePreview>),
    // AddPage,
    // PageAdded(PagePreview),
}

impl Preview {
    /// Creates a new [`Preview`] instance with no handle and non-inverted colors.
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: AbsoluteOffset::default(),
            viewport_h: DEFAULT_VIEWPORT_H,
            pages_to_render: (0, 0),
            pages: None,
        }
    }

    // makes a vec of (begin, end)
    // with `begin` the absolute position at the beginning of the page
    // and `end` the absolute position at the end of the page
    fn pages_positions(&self, document: &PagedDocument) -> Vec<(f32, f32)> {
        document
            .pages
            .iter()
            .scan(0.0_f32, |offset, page| {
                *offset += SPACING_BETWEEN_PAGES as f32;
                let begin = *offset;
                *offset += page.frame.height().to_pt() as f32 * self.zoom;
                let end = *offset;
                Some((begin, end))
            })
            .collect()
    }

    fn compute_pages_to_render(&self, document: &PagedDocument) -> (usize, usize) {
        let pages_positions = self.pages_positions(document);
        let start = pages_positions
            .iter()
            .take_while(|(_, e)| *e < self.offset.y)
            .count();
        let end = pages_positions
            .iter()
            .take_while(|(s, _)| *s < self.offset.y + self.viewport_h)
            .count();
        (start, end)
    }

    pub fn view(&self) -> Element<Message> {
        if let Some(pages) = &self.pages {
            let mut img_pages = vec![];
            for page in pages {
                img_pages.push(page.view(self.zoom));
            }
            let preview = scrollable(
                column(img_pages)
                    .spacing(SPACING_BETWEEN_PAGES)
                    .padding(15)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .direction(Direction::Both {
                vertical: Scrollbar::new().margin(2).width(5).scroller_width(4),
                horizontal: Scrollbar::new().margin(2).width(5).scroller_width(4),
            })
            .on_scroll(Message::Scrolled);

            column![
                // make prettier later
                row![
                    button("-").on_press(Message::DecrementZoom),
                    text(format!("{}%", (self.zoom * 100.0) as usize)),
                    button("+").on_press(Message::IncrementZoom)
                ]
                .align_y(Alignment::Center)
                .spacing(5),
                preview
            ]
            .align_x(Alignment::Center)
            .into()
        } else {
            column![].into()
        }
    }

    pub fn update(&mut self, message: Message, document: &PagedDocument) -> Task<Message> {
        match message {
            Message::IncrementZoom => {
                // max zoom at 250%
                if self.zoom < 2.5 {
                    self.zoom = (self.zoom + 0.15).min(3.0);
                    return Task::done(Message::ReloadPages);
                }
                Task::none()
            }
            Message::DecrementZoom => {
                // min zoom at 10%
                if self.zoom > 0.1 {
                    self.zoom = (self.zoom - 0.15).max(0.1);
                    return Task::done(Message::ReloadPages);
                }
                Task::none()
            }
            Message::Scrolled(viewport) => {
                let new_offset = viewport.absolute_offset();
                let new_viewport_h = viewport.bounds().height;
                let has_to_reload = (new_offset.y - self.offset.y).abs() > SCROLL_RELOAD_DELAY;
                if has_to_reload {
                    self.offset = new_offset;
                    self.viewport_h = new_viewport_h;
                    return Task::done(Message::ReloadPages);
                }
                Task::none()
            }
            Message::ReloadPages => {
                let new_index = self.compute_pages_to_render(document);
                println!("index: {:?}", new_index);
                if new_index != self.pages_to_render {
                    self.pages_to_render = new_index;
                    println!("perform reload");
                    return Task::perform(
                        fetch(self.pages_to_render, self.zoom, document.clone()),
                        Message::PagesReloaded,
                    );
                }
                Task::none()
            }
            Message::PagesReloaded(pages) => {
                self.pages = Some(pages.clone());
                println!("reloaded");
                Task::none()
            }
        }
    }
}

// compute rendering of viewable pages
async fn fetch(
    pages_to_render: (usize, usize),
    zoom: f32,
    document: PagedDocument,
) -> Vec<PagePreview> {
    document
        .pages
        .iter()
        .enumerate()
        .map(|(i, page)| {
            if i >= pages_to_render.0 && i <= pages_to_render.1 {
                PagePreview::fetch(page, zoom)
            } else {
                PagePreview::empty(page)
            }
        })
        .collect()
}
