use std::ops::Range;
use iced_graphics::{
    triangle::{Mesh2D, Vertex2D},
    Backend, Font, HorizontalAlignment, VerticalAlignment,
    Primitive, Size, Vector, Renderer, backend::Text as BackendWithText,
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

const CURSOR_MESH: (&[Vertex2D], &[u32]) = (&[
    Vertex2D { position: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [2.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [0.0, 6.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [2.0, 6.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [2.0, 4.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [24.0, 4.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [24.0, 6.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [24.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [26.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
    Vertex2D { position: [26.0, 6.0], color: [1.0, 1.0, 1.0, 1.0] },
], &[
    0, 1, 3,
    0, 2, 3,
    4, 5, 6,
    4, 3, 6,
    7, 8, 9,
    7, 6, 9,
]);


const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
const OFFSET_REFERENCE: &'static str = "00000000";
const BYTE_COLUMNS: &'static str = "00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F";
const LINE_SPACING: f32 = 8.0;
const LARGE_BOUNDS: Size<f32> = Size::new(640.0, 480.0);
const ASCII_RANGE: Range<u8> = 32..128;

impl<B: Backend + BackendWithText> hexview::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        _cursor_position: Point,
        style_sheet: &Self::Style,
        text_size: f32,
        column_count: usize,
        keyboard_focus: bool,
        cursor: usize,
        test_offset: f32,
        data: &[u8],
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
        let line_count = (data.len() as f32 / column_count as f32).ceil() as usize;
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
            let upper_bound = (lower_bound + column_count).min(data.len());
            let data_slice = &data[lower_bound..upper_bound];
            let byte_count = data_slice.len();
            let mut np_bytes = String::new();
            let mut np_ascii = String::new();
            let has_non_printable = data_slice.iter().any(|b| !ASCII_RANGE.contains(b));
            let np_have_color = style.non_printable_color.is_some();
            let bytes = data_slice
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, b)| {
                    let high = HEX_CHARS[(b >> 4) as usize] as char;
                    let low = HEX_CHARS[(b & 0xF) as usize] as char;

                    if ASCII_RANGE.contains(b) {
                        acc.push(high);
                        acc.push(low);

                        if has_non_printable && np_have_color {
                            np_bytes.push_str("  ");
                        }
                    } else {
                        acc.push_str("  ");

                        if np_have_color {
                            np_bytes.push(high);
                            np_bytes.push(low);
                        } else {
                            acc.push(high);
                            acc.push(low);
                        }
                    }

                    if i != 15 {
                        acc.push(' ');

                        if has_non_printable && np_have_color {
                            np_bytes.push(' ');
                        }
                    }

                    acc
                });
            let ascii = data_slice
                .iter()
                .fold(String::new(), |mut acc, b| {
                    if ASCII_RANGE.contains(b) {
                        acc.push(*b as char);

                        if has_non_printable && np_have_color {
                            np_ascii.push(' ');
                        }
                    } else {
                        if np_have_color {
                            acc.push(' ');
                            np_ascii.push('.');
                        } else {
                            acc.push('.');
                        }
                    }

                    acc
                });

            let line_x = bounds_pos.0 + 10.0;
            let line_y = bounds_pos.1 + data_y + i as f32 * (text_size + LINE_SPACING);

            let mut primitives = vec![
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
                    color: style.data_color,
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
                    color: style.data_color,
                    size: text_size,
                    font: HACK_REGULAR,
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                },
            ];
            let non_printable_opt = |c| if has_non_printable { Some(c) } else { None };

            if let Some(color) = style.non_printable_color.and_then(non_printable_opt) {
                primitives.push(Primitive::Text {
                    color,
                    content: np_bytes,
                    bounds: Rectangle {
                        x: bounds_pos.0 + right_of_offset + 20.0,
                        y: line_y,
                        width: bytes_width,
                        height: text_size,
                    },
                    size: text_size,
                    font: HACK_REGULAR,
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                });
                primitives.push(Primitive::Text {
                    color,
                    content: np_ascii,
                    bounds: Rectangle {
                        x: bounds_pos.0 + right_of_bytes + 20.0,
                        y: line_y,
                        width: text_size * byte_count as f32,
                        height: text_size,
                    },
                    size: text_size,
                    font: HACK_REGULAR,
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                });
            }

            Primitive::Group {
                primitives,
            }
        }).collect();

        let line = cursor / column_count;
        let line_offset = cursor % column_count;
        let line_str = linegroup_bytes_str(&lines[line]);

        let byte_offset = text_width(
            self.backend(),
            HACK_REGULAR,
            text_size,
            &line_str[0..(line_offset * 3)],
        );

        let cursor_mesh_pos = [
            right_of_offset + 17.0 + byte_offset - 2.0,
            10.0 + text_size + LINE_SPACING + 12.0 + ((text_size + LINE_SPACING) * (cursor / column_count) as f32),
        ];

        let cursor_mesh = Mesh2D {
            vertices: CURSOR_MESH.0.iter().map(|v| {
                Vertex2D {
                    position: v.position,
                    color: style.cursor_color.into_linear(),
                }
            }).collect::<Vec<_>>(),
            indices: CURSOR_MESH.1.to_vec(),
        };

        let cursor_prim = Primitive::Translate {
            translation: Vector::new(bounds_pos.0, bounds_pos.1) + cursor_mesh_pos.into(),
            content: Box::new(Primitive::Mesh2D {
                buffers: cursor_mesh,
                size: Size::new(
                    CURSOR_MESH.0[9].position[0],
                    CURSOR_MESH.0[9].position[1],
                ),
            }),
        };

        let debug_text = format!(
            "text_size: {}\n\
             keyboard_focus: {}\n\
             cursor: {}\n\
             cursor_position: ({}, {})\n\
             bytes length: {}\n\
             byte_offset: {}\n\
             test_offset: {}\n\
             bounds: ({}, {}) {}x{}",
            text_size, keyboard_focus, cursor,
            cursor_mesh_pos[0], cursor_mesh_pos[1],
            data.len(), byte_offset, test_offset,
            bounds_pos.0, bounds_pos.1, bounds_size.0,
            bounds_size.1,
        );

        let debug_line_count = debug_text
            .chars()
            .fold(1, |mut acc, c| {
                if c == '\n' {
                    acc += 1;
                }

                acc
            });

        let debug_info = group(vec![
            Primitive::Text {
                content: debug_text,
                bounds: Rectangle {
                    x: bounds_pos.0 + right_of_bytes + 20.0,
                    y: bounds_pos.1 + data_y + line_count as f32 * (text_size + LINE_SPACING),
                    width: 400.0,
                    height: text_size * debug_line_count as f32,
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
                cursor_prim,
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

fn linegroup_bytes_str(group: &Primitive) -> &str {
    if let Primitive::Group { primitives } = group {
        if let Primitive::Text { content, .. } = &primitives[1] {
            content
        } else {
            unreachable!();
        }
    } else {
        unreachable!();
    }
}
