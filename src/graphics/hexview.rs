use iced_graphics::{
    Backend, Font, HorizontalAlignment, VerticalAlignment,
    Primitive, Size, Renderer, backend::Text as BackendWithText,
};
use iced_native::{mouse, Background, Color, Point, Rectangle};
use crate::native::hexview;
pub use crate::style::hexview::{Style, StyleSheet};

macro_rules! load_font {
    ($p:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/fonts/",
            $p,
        ))
    };
}

const HACK_REGULAR: Font = Font::External {
    name: "Hack Regular",
    bytes: load_font!("hack-regular.ttf"),
};
const HACK_BOLD: Font = Font::External {
    name: "Hack Bold",
    bytes: load_font!("hack-bold.ttf"),
};

const TEST_DATA: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed \
                           interdum massa interdum gravida gravida. Nam ullamcorper.";
const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
const OFFSET_REFERENCE: &'static str = "00000000";
const BYTE_COLUMNS: &'static str = "00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F";
const LINE_SPACING: f32 = 9.0;
const LARGE_BOUNDS: Size<f32> = Size::new(640.0, 480.0);

impl<B: Backend + BackendWithText> hexview::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        _cursor_position: Point,
        style_sheet: &Self::Style,
        text_size: f32,
        column_count: usize,
    ) -> Self::Output {

        let style = style_sheet.active();
        let bounds_pos = (bounds.x.floor(), bounds.y.floor());
        let bounds_size = (bounds.width.floor(), bounds.height.floor());
        let back = Primitive::Quad {
            bounds: Rectangle {
                x: bounds_pos.0,
                y: bounds_pos.1,
                width: bounds_size.0,
                height: bounds_size.1,
            },
            background: Background::Color(style.background_color),
            border_radius: 0,
            border_width: 0,
            border_color: Color::BLACK,
        };
        let line_count = (TEST_DATA.len() as f32 / column_count as f32).ceil() as usize;
        let data_y = 10.0 + text_size + LINE_SPACING;

        let offset_width = text_width(self.backend(), HACK_BOLD, text_size, OFFSET_REFERENCE);
        let bytes_width = text_width(self.backend(), HACK_BOLD, text_size, &BYTE_COLUMNS[..(column_count * 3 - 1)]);
        let right_of_offset = 10.0 + offset_width;
        let right_of_bytes = right_of_offset + 20.0 + bytes_width;

        let offset_separator = Primitive::Quad {
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_offset + 10.0,
                y: bounds_pos.1 + 10.0,
                width: 0.5,
                height: bounds_size.1 - 20.0,
            },
            background: Background::Color(style.line_color),
            border_radius: 0,
            border_width: 0,
            border_color: Color::BLACK,
        };

        let bytes_separator = Primitive::Quad {
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_bytes + 10.0,
                y: bounds_pos.1 + 10.0,
                width: 0.5,
                height: bounds_size.1 - 20.0,
            },
            background: Background::Color(style.line_color),
            border_radius: 0,
            border_width: 0,
            border_color: Color::BLACK,
        };

        let byte_columns = Primitive::Text {
            content: BYTE_COLUMNS[0..(column_count as usize * 3 - 1)].into(),
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_offset + 20.0,
                y: bounds_pos.1 + 10.0,
                width: bytes_width,
                height: text_size,
            },
            color: style.offset_color,
            size: text_size,
            font: HACK_BOLD,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        };

        let ascii_hex_chars = std::str::from_utf8(&HEX_CHARS[0..column_count]).unwrap();
        let ascii_width = text_width(self.backend(), HACK_BOLD, text_size, ascii_hex_chars);
        let ascii_columns = Primitive::Text {
            content: ascii_hex_chars.into(),
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_bytes + 20.0,
                y: bounds_pos.1 + 10.0,
                width: ascii_width,
                height: text_size,
            },
            color: style.offset_color,
            size: text_size,
            font: HACK_BOLD,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        };

        let lines: Vec<Primitive> = (0..line_count).map(|i| {
            let lower_bound = column_count * i;
            let upper_bound = (lower_bound + column_count).min(TEST_DATA.len());
            let data_slice = &TEST_DATA[lower_bound..upper_bound];
            let byte_count = data_slice.len();
            let bytes = data_slice
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, b)| {
                    acc.push(HEX_CHARS[(b >> 4) as usize] as char);
                    acc.push(HEX_CHARS[(b & 0xF) as usize] as char);

                    if i != 15 {
                        acc.push(' ');
                    }

                    acc
                });
            let ascii = std::str::from_utf8(data_slice)
                .unwrap()
                .into();
            let line_x = bounds_pos.0 + 10.0;
            let line_y = bounds_pos.1 + data_y + i as f32 * (text_size + LINE_SPACING);

            Primitive::Group {
                primitives: vec![
                    // Offset
                    Primitive::Text {
                        content: format!("{:08X}", i * 16),
                        bounds: Rectangle {
                            x: line_x,
                            y: line_y,
                            width: offset_width,
                            height: text_size,
                        },
                        color: style.offset_color,
                        size: text_size,
                        font: HACK_BOLD,
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                    },

                    // Bytes
                    Primitive::Text {
                        content: bytes,
                        bounds: Rectangle {
                            x: bounds_pos.0 + right_of_offset + 20.0,
                            y: line_y,
                            width: bytes_width,
                            height: text_size,
                        },
                        color: style.offset_color,
                        size: text_size,
                        font: HACK_REGULAR,
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                    },

                    // Ascii
                    Primitive::Text {
                        content: ascii,
                        bounds: Rectangle {
                            x: bounds_pos.0 + right_of_bytes + 20.0,
                            y: line_y,
                            width: text_size * byte_count as f32,
                            height: text_size,
                        },
                        color: style.offset_color,
                        size: text_size,
                        font: HACK_REGULAR,
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                    },
                ],
            }
        }).collect();

        let debug_info = group(vec![
            Primitive::Text {
                content: format!("text_size: {}", text_size),
                bounds: Rectangle {
                    x: bounds_pos.0 + right_of_bytes + 20.0,
                    y: bounds_pos.1 + data_y + line_count as f32 * (text_size + LINE_SPACING),
                    width: 400.0,
                    height: text_size,
                },
                color: Color::from_rgb(1.0, 0.0, 0.0),
                size: text_size,
                font: HACK_REGULAR,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
            },
        ]);

        (
            group(vec![
                back,
                offset_separator,
                bytes_separator,
                byte_columns,
                ascii_columns,
                group(lines),
                debug_info,
            ]),
            mouse::Interaction::default(),
        )
    }
}

fn group(primitives: Vec<Primitive>) -> Primitive {
    Primitive::Group {
        primitives,
    }
}

fn text_width<B: BackendWithText>(backend: &B, font: Font, size: f32, text: &str) -> f32 {
    backend.measure(text, size, font, LARGE_BOUNDS).0
}
