use iced::advanced::layout::Node;
use iced::advanced::widget::tree::Tree;
use iced::advanced::{
    self, layout, overlay, renderer, widget, Clipboard, Layout, Shell, Widget,
};
use iced::mouse::Cursor;
use iced::{
    keyboard, mouse, touch, Element, Event, Length, Padding, Point, Rectangle, Size,
    Vector,
};

pub fn menu<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    overlay: impl Into<Element<'a, Message, Theme, Renderer>>,
    expanded: bool,
) -> Menu<'a, Message, Theme, Renderer> {
    Menu {
        base: base.into(),
        overlay: overlay.into(),
        anchor: Anchor::default(),
        expanded,
        on_dismiss: None,
        overlay_padding: Padding::ZERO,
    }
}

pub struct Menu<'a, Message, Theme, Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    overlay: Element<'a, Message, Theme, Renderer>,
    anchor: Anchor,
    expanded: bool,
    on_dismiss: Option<Message>,
    overlay_padding: Padding,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Anchor {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl<'a, Message, Theme, Renderer> Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: advanced::Renderer + 'a,
{
    pub fn anchor(
        mut self,
        anchor: Anchor,
    ) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn on_dismiss(
        mut self,
        on_dismiss: Message,
    ) -> Self {
        self.on_dismiss = Some(on_dismiss);
        self
    }

    // Apply padding inside total viewport
    pub fn overlay_padding(
        mut self,
        overlay_padding: impl Into<Padding>,
    ) -> Self {
        self.overlay_padding = overlay_padding.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: advanced::Renderer + 'a,
{
    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.base.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.base),
            widget::Tree::new(&self.overlay),
        ]
    }

    fn diff(
        &self,
        tree: &mut widget::Tree,
    ) {
        tree.diff_children(&[&self.base, &self.overlay]);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.base.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if !self.expanded {
            return self.base.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        Some(overlay::Element::new(Box::new(MenuOverlay::new(
            &mut tree.children[1],
            &mut self.overlay,
            layout.bounds(),
            layout.position() + translation,
            self.overlay_padding,
            *viewport,
            self.anchor,
            self.on_dismiss.clone(),
        ))))
    }
}

impl<'a, Message, Theme: 'a, Renderer> From<Menu<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: advanced::Renderer + 'a,
{
    fn from(menu: Menu<'a, Message, Theme, Renderer>) -> Self {
        Element::new(menu)
    }
}

struct MenuOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
{
    state: &'b mut Tree,
    element: &'b mut Element<'a, Message, Theme, Renderer>,
    underlay_bounds: Rectangle,
    position: Point,
    padding: Padding,
    viewport: Rectangle,
    anchor: Anchor,
    on_dismiss: Option<Message>,
}

impl<'a, 'b, Message, Theme, Renderer> MenuOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    #[allow(clippy::too_many_arguments)]
    fn new(
        state: &'b mut Tree,
        element: &'b mut Element<'a, Message, Theme, Renderer>,
        underlay_bounds: Rectangle,
        position: Point,
        padding: Padding,
        viewport: Rectangle,
        anchor: Anchor,
        on_dismiss: Option<Message>,
    ) -> Self {
        Self {
            state,
            element,
            underlay_bounds,
            position,
            padding,
            viewport,
            anchor,
            on_dismiss,
        }
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for MenuOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
    ) -> Node {
        let viewport = Rectangle::with_size(bounds).shrink(self.padding);

        let overlay_layout = self.element.as_widget_mut().layout(
            self.state,
            renderer,
            &layout::Limits::new(Size::ZERO, bounds),
        );

        let bounds = Rectangle::new(self.position, overlay_layout.size());

        // Positionnig stuff
        let x_center =
            self.position.x + (self.underlay_bounds.width - bounds.width) / 2.0;
        let y_center =
            self.position.y + (self.underlay_bounds.height - bounds.height) / 2.0;

        let mut overlay_bounds = {
            let offset = match self.anchor {
                Anchor::Bottom => Vector::new(x_center, self.position.y - bounds.height),
                Anchor::Top => {
                    Vector::new(x_center, self.position.y + self.underlay_bounds.height)
                }
                Anchor::Right => Vector::new(self.position.x - bounds.width, y_center),
                Anchor::Left => {
                    Vector::new(self.position.x + self.underlay_bounds.width, y_center)
                }
            };

            Rectangle {
                x: offset.x,
                y: offset.y,
                width: bounds.width,
                height: bounds.height,
            }
        };

        if overlay_bounds.x < viewport.x {
            overlay_bounds.x = viewport.x;
        } else if viewport.x + viewport.width < overlay_bounds.x + overlay_bounds.width {
            overlay_bounds.x = viewport.x + viewport.width - overlay_bounds.width;
        }

        if overlay_bounds.y < viewport.y {
            overlay_bounds.y = viewport.y;
        } else if viewport.y + viewport.height < overlay_bounds.y + overlay_bounds.height
        {
            overlay_bounds.y = viewport.y + viewport.height - overlay_bounds.height;
        }

        overlay_layout.move_to(overlay_bounds.position())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let bounds = layout.bounds();
        self.element
            .as_widget()
            .draw(self.state, renderer, theme, style, layout, cursor, &bounds);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) {
        self.underlay_bounds = Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: self.underlay_bounds.width,
            height: self.underlay_bounds.height,
        };

        if let Some(on_dismiss) = &self.on_dismiss {
            match &event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    if key == &keyboard::Key::Named(keyboard::key::Named::Escape) {
                        shell.publish(on_dismiss.clone());
                    }
                }

                Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left | mouse::Button::Right,
                ))
                | Event::Mouse(mouse::Event::WheelScrolled { .. })
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if !cursor.is_over(layout.bounds())
                        && !cursor.is_over(self.underlay_bounds)
                    {
                        shell.publish(on_dismiss.clone());
                    }
                }

                _ => {}
            }
        }

        self.element.as_widget_mut().update(
            self.state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element
            .as_widget()
            .mouse_interaction(self.state, layout, cursor, &self.viewport, renderer)
    }
}
