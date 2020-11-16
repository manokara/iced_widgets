use iced_native::{
    keyboard, layout, mouse,
    Clipboard, Element, Event, Hasher, Layout, Length,
    Point, Rectangle, Size, Widget,
};
use std::{
    hash::Hash,
    marker::PhantomData,
};
use crate::graphics::hexview::{LINE_SPACING, MARGINS};

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
    bytes_hash: u64,
    keyboard_focus: bool,
    test_offset: f32,
    debug_enabled: bool,
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
        keyboard_focus: bool,
        cursor: usize,
        test_offset: f32,
        debug_enabled: bool,
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
            bytes_hash: 0,
            keyboard_focus: false,
            test_offset: 0.0,
            debug_enabled: false,
        }
    }
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        use std::hash::Hasher;

        let mut hasher = iced_native::Hasher::default();
        hasher.write(bytes);
        self.bytes_hash = hasher.finish();
        self.bytes = bytes.to_vec();
        self.cursor = 0;
    }

    pub fn set_column_count(&mut self, count: usize) {
        self.column_count = count.max(1);
    }

    pub fn set_text_size(&mut self, size: f32) {
        self.text_size = size.max(10.0);
    }

    pub fn set_keyboard_focus(&mut self, focus: bool) {
        self.keyboard_focus = focus;
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
        let limits = limits.width(Length::Fill);
        let max_width = limits.max().width;
        let rows = (self.state.bytes.len() as f32 / self.state.column_count as f32).ceil();
        let rows_size = (self.state.text_size + LINE_SPACING) * rows;

        // Vertical margins + top headers + rows
        let height = MARGINS.y * 2.0 + self.state.text_size + LINE_SPACING + rows_size;
        println!("height: {:?}", height);

        layout::Node::new(Size::new(max_width, height))
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        use keyboard::{Event as KeyboardEvent, KeyCode};
        use mouse::{Button as MouseButton, Event as MouseEvent};

        let bytes_len = self.state.bytes.len();
        let column_count = self.state.column_count;
        let cursor = self.state.cursor;
        let keyboard_focus = self.state.keyboard_focus;
        let test_offset = self.state.test_offset;
        let debug_enabled = self.state.debug_enabled;

        match event {
            Event::Mouse(MouseEvent::ButtonReleased(MouseButton::Left)) => {
                self.state.set_keyboard_focus(layout.bounds().contains(cursor_position));
            }

            Event::Keyboard(KeyboardEvent::KeyPressed { key_code, .. }) => {
                let line_start = cursor / column_count * column_count;
                let line_end = (line_start + column_count - 1).min(bytes_len - 1);
                let cursor_guard_left = cursor > 0 && keyboard_focus;
                let cursor_guard_right = if bytes_len > 0 {
                    self.state.cursor < bytes_len - 1 && keyboard_focus
                } else {
                    false
                };
                let cursor_guard_up = cursor >= column_count && keyboard_focus;
                let cursor_guard_down = bytes_len > 0 && keyboard_focus;
                let cursor_guard_home = cursor > line_start && keyboard_focus;
                let cursor_guard_end = cursor < line_end && keyboard_focus;
                let cursor_guard_pageup = cursor > 0 && keyboard_focus;
                let cursor_guard_pagedown = bytes_len > 0 && keyboard_focus;
                let test_offset_guard_left = test_offset > f32::MIN && debug_enabled;
                let test_offset_guard_right = test_offset < f32::MAX && debug_enabled;

                match key_code {
                    // Cursor movement
                    KeyCode::Left if cursor_guard_left => self.state.cursor -= 1,
                    KeyCode::Right if cursor_guard_right => self.state.cursor += 1,
                    KeyCode::Up if cursor_guard_up => self.state.cursor -= column_count,
                    KeyCode::Down if cursor_guard_down => {
                        if cursor + column_count <= bytes_len - 1 {
                            self.state.cursor += column_count;
                        } else {
                            self.state.cursor = bytes_len - 1;
                        }
                    },
                    KeyCode::Home if cursor_guard_home => self.state.cursor = line_start,
                    KeyCode::End if cursor_guard_end => self.state.cursor = line_end,
                    // TODO: Calculate pages based on visible lines
                    KeyCode::PageUp if cursor_guard_pageup => self.state.cursor = 0,
                    KeyCode::PageDown if cursor_guard_pagedown => {
                        if bytes_len > 0 {
                            self.state.cursor = bytes_len - 1;
                        }
                    },

                    // Test offset
                    KeyCode::Minus if test_offset_guard_left => self.state.test_offset -= 0.01,
                    KeyCode::Equals if test_offset_guard_right => self.state.test_offset += 0.01,

                    // Debug
                    KeyCode::D => self.state.debug_enabled = !debug_enabled,

                    _ => (),
                }
            }

            _ => (),
        }
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
            self.state.keyboard_focus,
            self.state.cursor,
            self.state.test_offset,
            self.state.debug_enabled,
            &self.state.bytes,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;

        std::any::TypeId::of::<Marker>().hash(state);
        self.state.bytes_hash.hash(state);
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
