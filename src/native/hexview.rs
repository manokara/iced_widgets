use iced_native::{
    keyboard, layout, mouse,
    Clipboard, Element, Event, Font, Hasher, Layout, Length,
    Point, Rectangle, Size, Widget,
};
use std::{
    hash::Hash,
    marker::PhantomData,
};
use crate::{
    core::clamp,
    graphics::hexview::{LINE_SPACING, MARGINS},
};

/// A view into a region of bytes
#[allow(missing_debug_implementations)]
pub struct Hexview<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    style: Renderer::Style,
    header_font: Font,
    data_font: Font,
    font_size: f32,
    column_count: u8,
    message: PhantomData<Message>,
}

/// The local state of an [`Hexview`].
#[derive(Debug)]
pub struct State {
    bytes: Vec<u8>,
    cursor: usize,
    bytes_hash: u64,
    keyboard_focus: bool,
    test_offset: f32,
    debug_enabled: bool,
    last_click: Option<mouse::click::Click>,
    last_click_pos: Option<Point>,
    is_dragging: bool,
    selection: Option<(usize, usize)>,
    mouse_pos: Point,
}

pub trait Renderer: iced_native::Renderer {
    type Style: Default;

    fn cursor_offset(
        &self,
        bounds: Rectangle,
        cursor_position: Point,
        font: Font,
        size: f32,
        column_count: usize,
        extend_line: bool,
        bytes: &[u8],
    ) -> Option<usize>;

    fn measure(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32);

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
        header_font: Font,
        data_font: Font,
        selection: &Option<(usize, usize)>,
        data: &[u8],
    ) -> Self::Output;
}

impl<'a, Message, Renderer: self::Renderer> Hexview<'a, Message, Renderer> {
    /// Creates a new Hexview.
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            style: Renderer::Style::default(),
            font_size: 17.0,
            header_font: Font::Default,
            data_font: Font::Default,
            column_count: 16,
            message: PhantomData,
        }
    }

    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// The size of the font used in the widget
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Font for column headers and offsets.
    pub fn header_font(mut self, font: Font) -> Self {
        self.header_font = font;
        self
    }

    /// Font for bytes and ascii.
    pub fn data_font(mut self, font: Font) -> Self {
        self.data_font = font;
        self
    }

    /// How many columns therer are in the view
    ///
    /// `count` will be clamped to a number in the range `1..=32`.
    pub fn column_count(mut self, count: u8) -> Self {
        self.column_count = clamp(count, 1, 32);
        self
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            cursor: 0,
            bytes_hash: 0,
            keyboard_focus: false,
            test_offset: 0.0,
            debug_enabled: false,
            last_click: None,
            last_click_pos: None,
            is_dragging: false,
            selection: None,
            mouse_pos: Point::new(0.0, 0.0),
        }
    }
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        use std::hash::Hasher;

        let mut hasher = iced_native::Hasher::default();
        hasher.write(bytes);
        self.bytes_hash = hasher.finish();
        self.bytes = bytes.to_vec();
        self.cursor = 0;
        self.selection = None;
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
        let rows = (self.state.bytes.len() as f32 / self.column_count as usize as usize as f32).ceil();
        let rows_size = (self.font_size + LINE_SPACING) * rows;

        // Vertical margins + top headers + rows
        let height = MARGINS.y * 2.0 + self.font_size + LINE_SPACING + rows_size;

        layout::Node::new(Size::new(max_width, height))
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _messages: &mut Vec<Message>,
        renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        use keyboard::{Event as KeyboardEvent, KeyCode};
        use mouse::{Button as MouseButton, Event as MouseEvent};

        let bytes_len = self.state.bytes.len();
        let column_count = self.column_count as usize;
        let cursor = self.state.cursor;
        let keyboard_focus = self.state.keyboard_focus;
        let test_offset = self.state.test_offset;
        let debug_enabled = self.state.debug_enabled;
        let last_click_pos = self.state.last_click_pos;

        match event {
            Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Left)) => {
                if !layout.bounds().contains(cursor_position) {
                    return;
                }

                self.state.is_dragging = true;

                let cursor_from_pos = renderer.cursor_offset(
                    layout.bounds(),
                    cursor_position,
                    self.data_font,
                    self.font_size,
                    column_count as usize as usize,
                    false,
                    &self.state.bytes,
                );
                println!("Cursor from pos: {:?}", cursor_from_pos);

                if let Some(cursor) = cursor_from_pos {
                    self.state.cursor = cursor;
                }

                self.state.selection = None;
                self.state.last_click_pos = Some(cursor_position);

                let click = mouse::Click::new(
                    cursor_position,
                    self.state.last_click,
                );

                self.state.last_click = Some(click);
            }

            Event::Mouse(MouseEvent::ButtonReleased(MouseButton::Left)) => {
                if let Some(pos) = self.state.last_click_pos.take() {
                    if cursor_position == pos {
                        self.state.selection = None;
                    }
                }

                self.state.is_dragging = false;
                self.state.set_keyboard_focus(layout.bounds().contains(cursor_position));
            }

            Event::Mouse(MouseEvent::CursorMoved { x, y }) => {
                if self.state.is_dragging {
                    let cursor_from_pos = renderer.cursor_offset(
                        layout.bounds(),
                        cursor_position,
                        self.data_font,
                        self.font_size,
                        column_count as usize as usize,
                        true,
                        &self.state.bytes,
                    );

                    if let Some(new_cursor) = cursor_from_pos {
                        if new_cursor < cursor {
                            self.state.selection = Some((new_cursor, cursor))
                        } else if new_cursor > cursor {
                            self.state.selection = Some((cursor, new_cursor))
                        } else {
                            self.state.selection = None;
                        }
                    }

                    println!("Selection: {:?}", self.state.selection);
                }
            }

            Event::Keyboard(KeyboardEvent::KeyPressed { key_code, .. }) => {
                let line_start = cursor / column_count as usize as usize * column_count as usize;
                let line_end = (line_start + column_count as usize - 1).min(bytes_len - 1);
                let cursor_guard_left = cursor > 0 && keyboard_focus;
                let cursor_guard_right = if bytes_len > 0 {
                    self.state.cursor < bytes_len - 1 && keyboard_focus
                } else {
                    false
                };
                let cursor_guard_up = cursor >= column_count as usize && keyboard_focus;
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
                    KeyCode::Up if cursor_guard_up => self.state.cursor -= column_count as usize,
                    KeyCode::Down if cursor_guard_down => {
                        if cursor + column_count as usize <= bytes_len - 1 {
                            self.state.cursor += column_count as usize;
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
                    KeyCode::D if keyboard_focus => self.state.debug_enabled = !debug_enabled,

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
            self.font_size,
            self.column_count as usize,
            self.state.keyboard_focus,
            self.state.cursor,
            self.state.test_offset,
            self.state.debug_enabled,
            self.header_font,
            self.data_font,
            &self.state.selection,
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
