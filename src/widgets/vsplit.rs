use iced::{
    advanced::{
        layout::{Limits, Node},
        overlay,
        renderer::Style,
        widget::{tree, Operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    event::Status,
    mouse::{self, Cursor, Interaction},
    widget::{rule, Rule},
    window::RedrawRequest,
    Element, Event, Length, Pixels, Point, Rectangle, Renderer, Size, Theme, Vector,
};
use std::fmt::{Debug, Formatter};

/// Layout strategies for the [`VSplit`] widget.
#[derive(Clone, Copy, Debug, Default)]
pub enum Strategy {
    /// Position is a relative ratio (0.0 to 1.0) of the container width.
    #[default]
    Relative,
    /// Position is an absolute distance from the left edge.
    Left,
    /// Position is an absolute distance from the right edge.
    Right,
}

#[derive(Default)]
struct State {
    dragging: bool,
    hovering: bool,
}

/// A custom vertical split widget that divides the screen into two resizable panels.
///
/// `VSplit` displays two child widgets side by side with a draggable vertical separator ([`Rule`])
/// between them, allowing users to adjust the width ratio interactively.
///
/// # Example
/// ```rust
/// let split = VSplit::new(left_view, right_view)
///     .split_at(0.3)
///     .strategy(Strategy::Relative)
///     .on_resize(Message::SplitMoved);
/// ```
///
/// # Features
/// - Three layout strategies: [`Strategy::Relative`], [`Strategy::Left`], [`Strategy::Right`]
/// - Draggable vertical [`Rule`] as a divider
/// - Emits a message on resize
///
/// # Layout Strategies
/// - `Relative`: `split_at` is a normalized ratio `[0.0, 1.0]`
/// - `Left`: `split_at` is an absolute pixel offset from the left
/// - `Right`: `split_at` is an absolute offset from the right edge
pub struct VSplit<'a, Message> {
    children: [Element<'a, Message>; 3],
    split_at: f32,
    strategy: Strategy,
    rule_width: f32,
    on_resize: Option<fn(f32) -> Message>,
}

impl<Message> Debug for VSplit<'_, Message> {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("VSplit")
            .field("split_at", &self.split_at)
            .field("strategy", &self.strategy)
            .field("rule_width", &self.rule_width)
            .finish_non_exhaustive()
    }
}

impl<'a, Message> VSplit<'a, Message>
where
    Message: 'a,
{
    /// Creates a new [`VSplit`] with two child widgets (left and right).
    ///
    /// The default `split_at` is `0.5`, with strategy [`Strategy::Relative`].
    pub fn new(
        left: impl Into<Element<'a, Message>>,
        right: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            children: [left.into(), Rule::vertical(11.0).into(), right.into()],
            split_at: 0.5,
            strategy: Strategy::default(),
            rule_width: 11.0,
            on_resize: None,
        }
    }

    /// Sets the position of the split:
    /// - If `strategy` is `Relative`, `split_at` should be in `[0.0, 1.0]`
    /// - If `Left` or `Right`, it's interpreted in absolute pixels
    pub fn split_at(
        mut self,
        split_at: f32,
    ) -> Self {
        self.split_at = split_at;
        self
    }

    /// Defines the layout strategy (see [`Strategy`]).
    pub fn strategy(
        mut self,
        strategy: Strategy,
    ) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets the thickness of the split rule, in pixels.
    pub fn rule_width(
        mut self,
        rule_width: impl Into<Pixels>,
    ) -> Self {
        self.rule_width = rule_width.into().0;
        self.children[1] = Rule::vertical(self.rule_width).into();
        self
    }

    /// Registers a callback message when the split position is updated via drag.
    pub fn on_resize(
        mut self,
        on_resize: fn(f32) -> Message,
    ) -> Self {
        self.on_resize = Some(on_resize);
        self
    }

    /// Allows customizing the rule’s style using a closure with access to the theme.
    pub fn style(
        mut self,
        style: impl Fn(&Theme) -> rule::Style + 'a,
    ) -> Self {
        self.children[1] = Rule::vertical(self.rule_width).style(style).into();
        self
    }
}

/// Implements the full [Widget] trait:
/// dragging logic is handled in `on_event()`, where the resize callback is triggered
/// on cursor move while dragging.
impl<Message> Widget<Message, Theme, Renderer> for VSplit<'_, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        let max_limits = limits.max();

        let left_width = match self.strategy {
            Strategy::Relative => max_limits
                .width
                .mul_add(self.split_at, -self.rule_width / 2.0)
                .floor(),
            Strategy::Left => self.split_at,
            Strategy::Right => max_limits.width - self.split_at - self.rule_width,
        }
        .min(max_limits.width)
        .max(0.0);

        let left_limits = Limits::new(
            Size::new(0.0, 0.0),
            Size::new(left_width, max_limits.height),
        );

        let right_width = max_limits.width - left_width - self.rule_width;
        let right_limits = Limits::new(
            Size::new(0.0, 0.0),
            Size::new(right_width, max_limits.height),
        );

        let children = vec![
            self.children[0].as_widget().layout(
                &mut tree.children[0],
                renderer,
                &left_limits,
            ),
            self.children[1]
                .as_widget()
                .layout(&mut tree.children[1], renderer, limits)
                .translate(Vector::new(left_width, 0.0)),
            self.children[2]
                .as_widget()
                .layout(&mut tree.children[2], renderer, &right_limits)
                .translate(Vector::new(left_width + self.rule_width, 0.0)),
        ];

        Node::with_children(max_limits, children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
            .for_each(|((child, tree), layout)| {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
            });
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(
        &self,
        tree: &mut Tree,
    ) {
        tree.diff_children(&self.children);
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> Status {
        let mut event_status = Status::Ignored;

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        if let Event::Mouse(event) = event {
            match event {
                mouse::Event::ButtonPressed(mouse::Button::Left)
                    if cursor.is_over(layout.children().nth(1).unwrap().bounds()) =>
                {
                    state.dragging = true;
                    event_status = Status::Captured;
                }
                mouse::Event::CursorMoved {
                    position: Point { x, .. },
                    ..
                } => {
                    if state.dragging {
                        if let Some(on_resize) = self.on_resize {
                            let relative_pos = (x - bounds.x - self.rule_width / 2.0)
                                .clamp(0.0, bounds.width);
                            let split_at = match self.strategy {
                                Strategy::Relative => relative_pos / bounds.width,
                                Strategy::Left => relative_pos,
                                Strategy::Right => {
                                    bounds.width - relative_pos - self.rule_width
                                }
                            };
                            shell.publish(on_resize(split_at));
                            event_status = Status::Captured;
                        }
                    } else if state.hovering
                        != cursor.is_over(layout.children().nth(1).unwrap().bounds())
                    {
                        state.hovering ^= true;
                        shell.request_redraw(RedrawRequest::NextFrame);
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) if state.dragging => {
                    state.dragging = false;
                    event_status = Status::Captured;
                }
                _ => {}
            }
        }
        let child_status = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(Status::Ignored, Status::merge);

        Status::merge(event_status, child_status)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        let state = tree.state.downcast_ref::<State>();

        let interaction = self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default();

        if interaction == Interaction::default() && (state.dragging || state.hovering) {
            Interaction::ResizingHorizontally
        } else {
            interaction
        }
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, translation)
    }
}

/// Allows [`VSplit`] to be used directly as an [`Element`].
impl<'a, Message> From<VSplit<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: VSplit<'a, Message>) -> Self {
        Self::new(value)
    }
}
