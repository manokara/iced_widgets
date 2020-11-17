use iced::{
    pick_list, scrollable, slider, Align, Checkbox, Column, Container, Element, Font, Length,
    PickList, Radio, Row, Sandbox, Scrollable, Settings, Slider, Text,
};
use iced_widgets::{native::hexview, style::hexview as hexview_style};

macro_rules! load_data {
    ($p:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/", $p,))
    };
}

const NOTO_REGULAR: Font = Font::External {
    name: "Noto Regular",
    bytes: load_data!("fonts/noto-regular.ttf"),
};

const NOTO_BOLD: Font = Font::External {
    name: "Noto Bold",
    bytes: load_data!("fonts/noto-bold.ttf"),
};

const HACK_REGULAR: Font = Font::External {
    name: "Hack Regular",
    bytes: load_data!("fonts/hack-regular.ttf"),
};
const HACK_BOLD: Font = Font::External {
    name: "Hack Bold",
    bytes: load_data!("fonts/hack-bold.ttf"),
};

const LOREM_IPSUM: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed \
                             interdum massa interdum gravida gravida. Nam ullamcorper.";
const TGA_IMAGE: &[u8] = load_data!("black_square.tga");
const PNG_IMAGE: &[u8] = load_data!("black_square.png");
const SAMPLE_OPTIONS: &[&'static str] = &["Lorem Ipsum", "TGA Image", "PNG Image"];
const FONT_OPTIONS: &[&'static str] = &["Default", "Noto Sans", "Hack"];

#[derive(Debug, Clone)]
pub enum Message {
    ColumnCount(u8),
    ThemeSelected(Theme),
    ContentSelected(&'static str),
    FontSelected(&'static str),
    HighlightNonPrintable(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

pub struct App {
    hexview_theme: Theme,
    content_name: &'static str,
    font_name: &'static str,
    hexview_fonts: (Font, Font),
    highlight_np: bool,
    hexview_columns: u8,
    hexview: hexview::State,
    column_slider: slider::State,
    content_list: pick_list::State<&'static str>,
    scrollable: scrollable::State,
    font_list: pick_list::State<&'static str>,
}

pub struct HexviewTheme {
    base: Theme,
    highlight_np: bool,
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        let mut hexview = hexview::State::new();
        hexview.set_bytes(LOREM_IPSUM);

        App {
            hexview,
            hexview_theme: Theme::Light,
            content_name: "Lorem Ipsum",
            font_name: "Default",
            hexview_fonts: (Font::Default, Font::Default),
            highlight_np: true,
            hexview_columns: 16,
            column_slider: slider::State::new(),
            content_list: pick_list::State::default(),
            scrollable: scrollable::State::new(),
            font_list: pick_list::State::default(),
        }
    }

    fn title(&self) -> String {
        format!("manokara's iced widgets - hexview")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::ColumnCount(n) => self.hexview_columns = n,
            Message::ThemeSelected(t) => self.hexview_theme = t,
            Message::ContentSelected(name) => {
                match name {
                    "Lorem Ipsum" => self.hexview.set_bytes(LOREM_IPSUM),
                    "TGA Image" => self.hexview.set_bytes(TGA_IMAGE),
                    "PNG Image" => self.hexview.set_bytes(PNG_IMAGE),
                    _ => (),
                }

                self.content_name = name;
            }
            Message::FontSelected(name) => {
                match name {
                    "Default" => self.hexview_fonts = (Font::Default, Font::Default),
                    "Noto Sans" => self.hexview_fonts = (NOTO_REGULAR, NOTO_BOLD),
                    "Hack" => self.hexview_fonts = (HACK_REGULAR, HACK_BOLD),
                    _ => (),
                }

                self.font_name = name;
            }
            Message::HighlightNonPrintable(b) => {
                self.highlight_np = b;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let column_slider = Slider::new(
            &mut self.column_slider,
            1.0..=32.0,
            self.hexview_columns as f32,
            |n| Message::ColumnCount(n.floor() as u8),
        )
        .width(Length::Units(64));

        let light_radio = Radio::new(
            Theme::Light,
            "Light",
            Some(self.hexview_theme),
            Message::ThemeSelected,
        );
        let dark_radio = Radio::new(
            Theme::Dark,
            "Dark",
            Some(self.hexview_theme),
            Message::ThemeSelected,
        );

        let content_list = PickList::new(
            &mut self.content_list,
            SAMPLE_OPTIONS,
            Some(self.content_name),
            Message::ContentSelected,
        );

        let font_list = PickList::new(
            &mut self.font_list,
            FONT_OPTIONS,
            Some(self.font_name),
            Message::FontSelected,
        );

        let highlight_ckb = Checkbox::new(
            self.highlight_np,
            "Highlight non-printable",
            Message::HighlightNonPrintable,
        );

        let row = Row::with_children(vec![
            Text::new("Column Count:").into(),
            column_slider.into(),
            Text::new(format!("{}", self.hexview_columns)).into(),
            Text::new("Theme:").into(),
            light_radio.into(),
            dark_radio.into(),
            Text::new("Font:").into(),
            font_list.into(),
            Text::new("Content:").into(),
            content_list.into(),
            highlight_ckb.into(),
        ])
        .align_items(Align::Center)
        .spacing(12)
        .padding(8);

        let hexview_theme = modify_theme(self.hexview_theme, self.highlight_np);
        let hexview = hexview::Hexview::new(&mut self.hexview)
            .style(hexview_theme)
            .data_font(self.hexview_fonts.0)
            .header_font(self.hexview_fonts.1)
            .column_count(self.hexview_columns);

        let scrollable = Scrollable::new(&mut self.scrollable).push(hexview);

        let column = Column::with_children(vec![row.into(), scrollable.into()]);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn modify_theme(base: Theme, highlight_np: bool) -> HexviewTheme {
    HexviewTheme { base, highlight_np }
}

impl Into<Box<dyn hexview_style::StyleSheet>> for Theme {
    fn into(self) -> Box<dyn hexview_style::StyleSheet> {
        match self {
            Theme::Light => Box::new(hexview_style::Light),
            Theme::Dark => Box::new(hexview_style::Dark),
        }
    }
}

impl hexview_style::StyleSheet for HexviewTheme {
    fn active(&self) -> hexview_style::Style {
        let base: Box<dyn hexview_style::StyleSheet> = self.base.into();
        let active = base.active();
        let non_printable_color = if self.highlight_np {
            active.non_printable_color
        } else {
            None
        };

        hexview_style::Style {
            non_printable_color,
            ..active
        }
    }
}
