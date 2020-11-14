use iced_native::{
    keyboard, layout, Clipboard, Element, Event, Hasher, Layout, Length, Point, Rectangle, Size,
    Widget,
};
use std::{
    hash::Hash,
    marker::PhantomData,
};

/// A view into a region of bytes
#[allow(missing_debug_implementations)]
pub struct Hexview<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    style: Renderer::Style,
    message: PhantomData<Message>,
}

/// The local state of an [`Hexview`].
#[derive(Debug)]
pub struct State {
    bytes: Vec<u8>,
    cursor: usize,
    text_size: f32,
    column_count: usize,
    changed_bytes: bool,
}

pub trait Renderer: iced_native::Renderer {
    type Style: Default;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        style: &Self::Style,
        text_size: f32,
        column_count: usize,
        data: &[u8],
    ) -> Self::Output;
}

impl<'a, Message, Renderer: self::Renderer> Hexview<'a, Message, Renderer> {
    /// Creates a new Hexview.
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            style: Renderer::Style::default(),
            message: PhantomData,
        }
    }

    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            cursor: 0,
            text_size: 17.0,
            column_count: 16,
            changed_bytes: false,
        }
    }
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        self.bytes = bytes.to_vec();
        self.changed_bytes = true;
    }

    pub fn set_column_count(&mut self, count: usize) {
        self.column_count = count.max(1);
    }

    pub fn set_text_size(&mut self, size: f32) {
        self.text_size = size.max(10.0);
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Hexview<'a, Message, Renderer>
where
    Renderer: self::Renderer
{
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Fill);
        let _rows = (self.state.bytes.len() as f32 / self.state.column_count as f32).ceil();
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            cursor_position,
            &self.style,
            self.state.text_size,
            self.state.column_count.max(1),
            &self.state.bytes,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;

        std::any::TypeId::of::<Marker>().hash(state);
        self.state.changed_bytes.hash(state);
    }
}

impl<'a, Message, Renderer> From<Hexview<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(hexview: Hexview<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(hexview)
    }
}
