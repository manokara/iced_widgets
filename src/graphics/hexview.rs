use std::ops::Range;
use iced_graphics::{
    triangle::{Mesh2D, Vertex2D},
    Backend, Font, HorizontalAlignment, VerticalAlignment,
    Primitive, Size, Vector, Renderer, backend::Text as BackendWithText,
};
use iced_native::{mouse, Background, Color, Point, Rectangle};
use crate::native::hexview;
pub use crate::style::hexview::{Style, StyleSheet};

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
const CURSOR_RIGHT_VERTEX: ([usize; 3], [usize; 2]) = ([5, 6, 7], [8, 9]);
const CURSOR_PADDING: f32 = 4.0;

pub const LINE_SPACING: f32 = 8.0;
pub const MARGINS: Vector = Vector::new(10.0, 10.0);
const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
const OFFSET_REFERENCE: &'static str = "00000000";
const BYTES_HEADER: &'static str = "00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F";
const LARGE_BOUNDS: Size<f32> = Size::new(640.0, 480.0);
const ASCII_RANGE: Range<u8> = 32..128;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpanType {
    Printable,
    NonPrintable,
}

#[derive(Clone, Copy, Debug)]
struct LineSpan {
    ty: SpanType,
    start: usize,
    end: usize,
}

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
        debug_enabled: bool,
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
        let data_y = MARGINS.y + text_size + LINE_SPACING;

        let offset_width = text_width(self.backend(), style.header_font, text_size, OFFSET_REFERENCE);
        let bytes_header_width = text_width(
            self.backend(),
            style.header_font,
            text_size,
            &BYTES_HEADER[..(column_count * 3 - 1)],
        );
        let right_of_offset = MARGINS.x + offset_width;
        let right_of_bytes_header = right_of_offset + MARGINS.x * 2.0 + bytes_header_width;

        let offset_separator = Primitive::Quad {
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_offset + MARGINS.x,
                y: bounds_pos.1 + MARGINS.y,
                width: 0.5,
                height: bounds_size.1 - MARGINS.y * 2.0,
            },
            background: Background::Color(style.line_color),
            border_radius: 0,
            border_width: 0,
            border_color: Color::BLACK,
        };

        let bytes_header = Primitive::Text {
            content: BYTES_HEADER[0..(column_count as usize * 3 - 1)].into(),
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_offset + MARGINS.x * 2.0,
                y: bounds_pos.1 + MARGINS.y,
                width: bytes_header_width,
                height: text_size,
            },
            color: style.offset_color,
            size: text_size,
            font: style.header_font,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        };

        let ascii_hex_chars = std::str::from_utf8(&HEX_CHARS[0..column_count]).unwrap();
        let ascii_width = text_width(self.backend(), style.header_font, text_size, ascii_hex_chars);
        let start_of_bytes = right_of_offset + MARGINS.x * 2.0;
        let mut byte_buffers = Vec::new();

        let lines: Vec<Primitive> = (0..line_count).map(|i| {
            let lower_bound = column_count * i;
            let upper_bound = (lower_bound + column_count).min(data.len());
            let data_slice = &data[lower_bound..upper_bound];
            let line_x = bounds_pos.0 + MARGINS.x;
            let line_y = bounds_pos.1 + data_y + i as f32 * (text_size + LINE_SPACING);
            let np_have_color = style.non_printable_color.is_some();

            let mut byte_spans = Vec::new();
            let mut ascii_spans = Vec::new();
            let mut np_control = false;
            let mut printable_offset = 0;
            let mut np_offset = 0;
            let mut data_x = start_of_bytes;

            // Generate hexpairs in spans that will be transformed to text later
            let byte_buffer = data_slice
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, b)| {
                    if ASCII_RANGE.contains(b) {
                        // Update printable offset and generate non-printable span
                        if np_control {
                            printable_offset = acc.len();

                            if printable_offset > 0 {
                                let ty = if np_have_color {
                                    SpanType::NonPrintable
                                } else {
                                    SpanType::Printable
                                };

                                byte_spans.push(LineSpan {
                                    ty,
                                    start: np_offset,
                                    end: printable_offset,
                                });
                            }

                            np_control = false;
                        }
                    } else {
                        // Update non-printable offset and generate printable span
                        if !np_control {
                            np_offset = acc.len();

                            if np_offset > 0 {
                                byte_spans.push(LineSpan {
                                    ty: SpanType::Printable,
                                    start: printable_offset,
                                    end: np_offset,
                                });
                            }

                            np_control = true;
                        }
                    }

                    let high = HEX_CHARS[(b >> 4) as usize] as char;
                    let low = HEX_CHARS[(b & 0xF) as usize] as char;

                    acc.push(high);
                    acc.push(low);

                    if i != 15 {
                        acc.push(' ');
                    }

                    acc
                });

            // Add last span
            let (start, ty) = if np_control {
                (np_offset, if np_have_color {
                    SpanType::NonPrintable
                } else {
                    SpanType::Printable
                })
            } else {
                (printable_offset, SpanType::Printable)
            };

            byte_spans.push(LineSpan {
                ty,
                start,
                end: byte_buffer.len(),
            });

            printable_offset = 0;
            np_offset = 0;

            // Generate the ASCII repesentation in spans that will be transformed to text later
            let ascii_buffer = data_slice
                .iter()
                .fold(String::new(), |mut acc, b| {
                    if ASCII_RANGE.contains(b) {
                        if np_control {
                            printable_offset = acc.len();

                            if printable_offset > 0 {
                                let ty = if np_have_color {
                                    SpanType::NonPrintable
                                } else {
                                    SpanType::Printable
                                };

                                ascii_spans.push(LineSpan {
                                    ty,
                                    start: np_offset,
                                    end: printable_offset,
                                });
                            }

                            np_control = false;
                        }

                        acc.push(*b as char);
                    } else {
                        if !np_control {
                            np_offset = acc.len();

                            if np_offset > 0 {
                                ascii_spans.push(LineSpan {
                                    ty: SpanType::Printable,
                                    start: printable_offset,
                                    end: np_offset,
                                });
                            }

                            np_control = true;
                        }

                        acc.push('.');
                    }

                    acc
                });

            // Add last span
            let (start, ty) = if np_control {
                (np_offset, if np_have_color {
                    SpanType::NonPrintable
                } else {
                    SpanType::Printable
                })
            } else {
                (printable_offset, SpanType::Printable)
            };

            ascii_spans.push(LineSpan {
                ty,
                start,
                end: ascii_buffer.len(),
            });

            // Join spans with the same type
            byte_spans.dedup_by(span_dedup);
            ascii_spans.dedup_by(span_dedup);

            let byte_prims = byte_spans
                .iter()
                .fold(Vec::new(), |mut acc, span| {
                    let content = byte_buffer[span.start..span.end].to_string();
                    let content_width = text_width(
                        self.backend(),
                        style.data_font,
                        text_size,
                        &content,
                    );
                    let color = match span.ty {
                        SpanType::Printable => style.data_color,
                        SpanType::NonPrintable => style.non_printable_color.unwrap(),
                    };

                    acc.push(Primitive::Text {
                        content,
                        color,
                        bounds: Rectangle {
                            x: bounds_pos.0 + data_x,
                            y: line_y,
                            width: content_width,
                            height: text_size,
                        },
                        size: text_size,
                        font: style.data_font,
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                    });

                    data_x += content_width + test_offset;
                    acc
                });

            data_x = right_of_bytes_header + MARGINS.x * 2.0;

            let ascii_prims = ascii_spans
                .iter()
                .fold(Vec::new(), |mut acc, span| {
                    let content = ascii_buffer[span.start..span.end].to_string();
                    let content_width = text_width(
                        self.backend(),
                        style.data_font,
                        text_size,
                        &content,
                    );
                    let color = match span.ty {
                        SpanType::Printable => style.data_color,
                        SpanType::NonPrintable => style.non_printable_color.unwrap(),
                    };

                    acc.push(Primitive::Text {
                        content,
                        color,
                        bounds: Rectangle {
                            x: bounds_pos.0 + data_x,
                            y: line_y,
                            width: content_width,
                            height: text_size,
                        },
                        size: text_size,
                        font: style.data_font,
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                    });

                    // FIXME: Why is the width over by one pixel?
                    // This seems to happen all the example fonts in the demo.
                    // The spans don't align without this.
                    data_x += content_width + test_offset;
                    acc
                });

            let primitives = vec![
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
                    font: style.header_font,
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                },

                // Bytes
                group(byte_prims),

                // Ascii
                group(ascii_prims),
            ];

            byte_buffers.push(byte_buffer);
            group(primitives)
        }).collect();

        let bytes_separator = Primitive::Quad {
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_bytes_header + MARGINS.x,
                y: bounds_pos.1 + MARGINS.y,
                width: 0.5,
                height: bounds_size.1 - MARGINS.y * 2.0,
            },
            background: Background::Color(style.line_color),
            border_radius: 0,
            border_width: 0,
            border_color: Color::BLACK,
        };

        let ascii_columns = Primitive::Text {
            content: ascii_hex_chars.into(),
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_bytes_header + MARGINS.x * 2.0,
                y: bounds_pos.1 + MARGINS.y,
                width: ascii_width,
                height: text_size,
            },
            color: style.offset_color,
            size: text_size,
            font: style.header_font,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        };

        let line = cursor / column_count;
        let line_offset = cursor % column_count;
        let line_str = &byte_buffers[line];

        let byte_offset = text_width(
            self.backend(),
            style.data_font,
            text_size,
            &line_str[0..(line_offset * 3)],
        );

        let pair_width = text_width(
            self.backend(),
            style.data_font,
            text_size,
            &line_str[(line_offset * 3)..(line_offset * 3 + 2)],
        );

        let cursor_width = pair_width + CURSOR_PADDING;

        let cursor_mesh_pos = [
            start_of_bytes + byte_offset + pair_width - pair_width / 2.0 - cursor_width / 2.0,
            MARGINS.y + text_size + LINE_SPACING + 12.0 + ((text_size + LINE_SPACING) * (cursor / column_count) as f32),
        ];

        let cursor_mesh = Mesh2D {
            vertices: CURSOR_MESH.0.iter().enumerate().map(|(i, v)| {
                let position = if CURSOR_RIGHT_VERTEX.0.contains(&i) {
                    [cursor_width - 2.0, v.position[1]]
                } else if CURSOR_RIGHT_VERTEX.1.contains(&i) {
                    [cursor_width, v.position[1]]
                } else {
                    v.position
                };

                Vertex2D {
                    position,
                    color: style.cursor_color.into_linear(),
                }
            }).collect::<Vec<_>>(),
            indices: CURSOR_MESH.1.to_vec(),
        };

        let cursor_size = Size::new(
            cursor_mesh.vertices[9].position[0],
            cursor_mesh.vertices[9].position[1],
        );

        let cursor_prim = Primitive::Translate {
            translation: Vector::new(bounds_pos.0, bounds_pos.1) + cursor_mesh_pos.into(),
            content: Box::new(Primitive::Mesh2D {
                buffers: cursor_mesh,
                size: cursor_size,
            }),
        };

        let debug_info = if debug_enabled {
            let debug_text = format!(
                "text_size: {}\n\
                keyboard_focus: {}\n\
                cursor: {}\n\
                cursor_position: ({}, {})\n\
                cursor_size: {}x{}\n\
                bytes length: {}\n\
                byte_offset: {}\n\
                test_offset: {}\n\
                bounds: ({}, {}) {}x{}",
                text_size, keyboard_focus, cursor,
                cursor_mesh_pos[0], cursor_mesh_pos[1],
                cursor_size.width, cursor_size.height,
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

            group(vec![
                Primitive::Text {
                    content: debug_text,
                    bounds: Rectangle {
                        x: bounds_pos.0 + MARGINS.x,
                        y: bounds_pos.1 + data_y + line_count as f32 * (text_size + LINE_SPACING),
                        width: 400.0,
                        height: text_size * debug_line_count as f32,
                    },
                    color: Color::from_rgb(1.0, 0.0, 0.0),
                    size: text_size,
                    font: style.data_font,
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                },
            ])
        } else {
            Primitive::None
        };

        (
            group(vec![
                back,
                offset_separator,
                bytes_separator,
                bytes_header,
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

fn span_dedup(a: &mut LineSpan, b: &mut LineSpan) -> bool {
    if a.ty == b.ty {
        b.end = a.end;
        true
    } else {
        false
    }
}
