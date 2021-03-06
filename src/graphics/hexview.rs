//! Renderer implementation for an [`Hexview`].
//!
//! [`Hexview`]: struct.Heview.html

use std::ops::Range;
use iced_graphics::{
    triangle::{Mesh2D, Vertex2D},
    Backend, Font, HorizontalAlignment, VerticalAlignment,
    Primitive, Size, Vector, Renderer, backend::Text as BackendWithText,
};
use iced_native::{mouse, Background, Color, Point, Rectangle};
use crate::{
    core::range_intersect,
    native::hexview,
    style::hexview as style,
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
const CURSOR_RIGHT_VERTEX: ([usize; 3], [usize; 2]) = ([5, 6, 7], [8, 9]);
const CURSOR_PADDING: f32 = 4.0;

/// Spacing between each row in an [`Hexview`].
///
/// [`Hexview`]: struct.Heview.html
pub const LINE_SPACING: f32 = 8.0;

/// General margins used in an [`Hexview`].
///
/// This constant is used both for padding the text and also for spacing the
/// section separators.
///
/// [`Hexview`]: struct.Heview.html
pub const MARGINS: Vector = Vector::new(10.0, 10.0);
const HEX_CHARS: &[u8] = b"0123456789ABCDEF\
                           0123456789ABCDEF";
const OFFSET_REFERENCE: &'static str = "00000000";
const BYTES_HEADER: &'static str = "00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F \
                                    10 11 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E 1F";
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
    type Style = Box<dyn style::StyleSheet>;

    fn cursor_offset(
        &self,
        bounds: Rectangle,
        cursor_position: Point,
        font: Font,
        text_size: f32,
        column_count: usize,
        extend_line: bool,
        bytes: &[u8],
    ) -> Option<usize> {
        let row_count = (bytes.len() as f32 / column_count as f32).ceil() as usize;

        let offset_width = self.measure(
            OFFSET_REFERENCE,
            text_size,
            font,
            bounds.size(),
        ).0;

        let bytes_header_slice = &BYTES_HEADER[0..(column_count * 3 - 1)];

        let bytes_width = self.measure(
            bytes_header_slice,
            text_size,
            font,
            bounds.size(),
        ).0;

        let start_of_bytes = Point::new(
            bounds.x.floor() + MARGINS.x + offset_width + MARGINS.x * 2.0,
            bounds.y.floor() + MARGINS.y + text_size + LINE_SPACING,
        );

        let size_of_bytes  = Size::new(
            bytes_width,
            start_of_bytes.y + row_count as f32 * (text_size + LINE_SPACING) - LINE_SPACING,
        );

        let bytes_bounds = Rectangle::new(start_of_bytes, size_of_bytes);

        if !bytes_bounds.contains(cursor_position) {
            return None;
        }

        let pair_size = self.measure(
            "FF",
            text_size,
            font,
            bounds.size(),
        );
        let space_width = self.measure(
            " ",
            text_size,
            font,
            bounds.size(),
        ).0;

        let mut cursor = None;

        for row in 0..row_count {
            let pair_y = (text_size + LINE_SPACING) * row as f32;

            let mut bytes_x = 0.0;
            let mut pair_positions = (0..column_count)
                .enumerate()
                .fold(Vec::new(), |mut acc, (i, _pair)| {
                    acc.push((Point::new(bytes_x, pair_y), pair_size.0));
                    bytes_x += pair_size.0 + if i != column_count -1 { space_width } else { 0.0 };
                    acc
                });

            if extend_line {
                pair_positions.push((Point::new(bytes_x, pair_y), space_width));
            }

            for (i, (position, width)) in pair_positions.iter().enumerate() {
                let bound  = Rectangle {
                    x: start_of_bytes.x + position.x,
                    y: start_of_bytes.y + position.y,
                    width: *width,
                    height: pair_size.0,
                };

                if bound.contains(cursor_position) {
                    cursor = Some(row * column_count + i);
                    break;
                }
            }
        }

        cursor
    }

    fn measure(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.backend().measure(content, size, font, bounds)
    }

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
        header_font: Font,
        data_font: Font,
        selection: &Option<(usize, usize)>,
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

        let offset_width = self.measure(
            OFFSET_REFERENCE,
            text_size,
            header_font,
            bounds.size(),
        ).0;

        let bytes_header_slice = &BYTES_HEADER[0..(column_count * 3 - 1)];
        let bytes_header_width = self.measure(
            bytes_header_slice,
            text_size,
            header_font,
            bounds.size(),
        ).0;

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
            content: bytes_header_slice.into(),
            bounds: Rectangle {
                x: bounds_pos.0 + right_of_offset + MARGINS.x * 2.0,
                y: bounds_pos.1 + MARGINS.y,
                width: bytes_header_width,
                height: text_size,
            },
            color: style.offset_color,
            size: text_size,
            font: header_font,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        };

        let ascii_hex_chars = std::str::from_utf8(&HEX_CHARS[0..column_count]).unwrap();

        let ascii_width = self.measure(
            ascii_hex_chars,
            text_size,
            header_font,
            bounds.size()
        ).0;

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

                    if i != data_slice.len() - 1 {
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
                    let content_width = self.measure(
                        &content,
                        text_size,
                        data_font,
                        bounds.size(),
                    ).0;
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
                        font: data_font,
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
                    let content_width = self.measure(
                        &content,
                        text_size,
                        data_font,
                        bounds.size(),
                    ).0;
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
                        font: data_font,
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                    });

                    // FIXME: Why is the width over by one pixel?
                    // This seems to happen all the example fonts in the demo.
                    // The spans don't align without this.
                    data_x += content_width + test_offset;
                    acc
                });

            let selection_prim = if let Some((start, end)) = selection {
                let selection = *start..*end;
                let row_range = lower_bound..upper_bound;
                let intersection = range_intersect(row_range, selection);

                if !intersection.is_empty() {
                    let intersection_size = intersection.end - intersection.start;
                    let start = intersection.start - lower_bound;
                    let end = start + intersection_size;

                    let (bytes_pre_slice, ascii_pre_slice) = if start > 0 {
                        (
                            &byte_buffer[0..(start * 3).min(byte_buffer.len())],
                            &ascii_buffer[0..start],
                        )
                    } else {
                        ("", "")
                    };

                    let bytes_slice = &byte_buffer[(start * 3)..(end * 3).min(byte_buffer.len())];
                    let ascii_slice = &ascii_buffer[start..end];
                    let bytes_width = self.measure(
                        bytes_slice,
                        text_size,
                        data_font,
                        bounds.size(),
                    ).0;
                    let ascii_width = self.measure(
                        ascii_slice,
                        text_size,
                        data_font,
                        bounds.size(),
                    ).0;

                    let bytes_x = self.measure(
                        bytes_pre_slice,
                        text_size,
                        data_font,
                        bounds.size(),
                    ).0;
                    let ascii_x = self.measure(
                        ascii_pre_slice,
                        text_size,
                        data_font,
                        bounds.size(),
                    ).0;

                    group(vec![
                        // Bytes
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: bounds_pos.0 + start_of_bytes + bytes_x,
                                y: line_y,
                                width: bytes_width,
                                height: text_size,
                            },
                            background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5)),
                            border_radius: 0,
                            border_width: 0,
                            border_color: Color::BLACK,
                        },

                        // Ascii
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: right_of_bytes_header + MARGINS.x * 2.0 + ascii_x,
                                y: line_y,
                                width: ascii_width,
                                height: text_size,
                            },
                            background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5)),
                            border_radius: 0,
                            border_width: 0,
                            border_color: Color::BLACK,
                        },
                    ])

                } else {
                    Primitive::None
                }
            } else {
                Primitive::None
            };

            let primitives = vec![
                // Offset
                Primitive::Text {
                    content: format!("{:08X}", i * column_count),
                    bounds: Rectangle {
                        x: line_x,
                        y: line_y,
                        width: offset_width,
                        height: text_size,
                    },
                    color: style.offset_color,
                    size: text_size,
                    font: header_font,
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                },

                // Bytes
                group(byte_prims),

                // Ascii
                group(ascii_prims),

                // Selection,
                selection_prim,

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
            font: header_font,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        };

        let line = cursor / column_count;
        let line_offset = cursor % column_count;
        let line_str = &byte_buffers[line];

        let byte_offset = self.measure(
            &line_str[0..(line_offset * 3)],
            text_size,
            data_font,
            bounds.size(),
        ).0;

        let pair_width = self.measure(
            &line_str[(line_offset * 3)..(line_offset * 3 + 2)],
            text_size,
            data_font,
            bounds.size(),
        ).0;

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
                 column_count: {}\n\
                 bytes_header_width: {}\n\
                 keyboard_focus: {}\n\
                 cursor: {}\n\
                 cursor_position: ({}, {})\n\
                 cursor_size: {}x{}\n\
                 bytes length: {}\n\
                 byte_offset: {}\n\
                 test_offset: {}\n\
                 bounds: ({}, {}) {}x{}",
                text_size, column_count, keyboard_focus, cursor,
                bytes_header_width,
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
                    font: data_font,
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

fn span_dedup(a: &mut LineSpan, b: &mut LineSpan) -> bool {
    if a.ty == b.ty {
        b.end = a.end;
        true
    } else {
        false
    }
}
