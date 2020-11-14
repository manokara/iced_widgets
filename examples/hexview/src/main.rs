use iced::{
    pick_list, scrollable, slider,
    Align, Column, Container, Element, Length, PickList,
    Radio, Row, Sandbox, Scrollable, Settings, Slider,
    Text,
};
use iced_widgets::{
    native::hexview,
    style::hexview as hexview_style,
};

macro_rules! load_data {
    ($p:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/examples/",
            $p,
        ))
    };
}

const LOREM_IPSUM: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed \
                             interdum massa interdum gravida gravida. Nam ullamcorper.";
const TGA_IMAGE: &[u8] = load_data!("black_square.tga");
const PNG_IMAGE: &[u8] = load_data!("black_square.png");
const SAMPLE_OPTIONS: &[&'static str] = &[
    "Lorem Ipsum",
    "TGA Image",
    "PNG Image",
];

#[derive(Debug, Clone)]
pub enum Message {
    ColumnCount(usize),
    ThemeSelected(Theme),
    ContentSelected(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

pub struct App {
    hexview_theme: Theme,
    content_name: &'static str,
    hexview: hexview::State,
    column_slider: slider::State,
    content_list: pick_list::State<&'static str>,
    scrollable: scrollable::State,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        let mut hexview =  hexview::State::new();
        hexview.set_bytes(LOREM_IPSUM);

        App {
            hexview,
            hexview_theme: Theme::Light,
            content_name: "Lorem Ipsum",
            column_slider: slider::State::new(),
            content_list: pick_list::State::default(),
            scrollable: scrollable::State::new(),
        }
    }

    fn title(&self) -> String {
        format!("manokara's iced widgets - hexview")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::ColumnCount(n) => self.hexview.set_column_count(n),
            Message::ThemeSelected(t) => self.hexview_theme = t,
            Message::ContentSelected(name) => {
                match name {
                    "Lorem Ipsum" => self.hexview.set_bytes(LOREM_IPSUM),
                    "TGA Image" => self.hexview.set_bytes(TGA_IMAGE),
                    "PNG Image" => self.hexview.set_bytes(PNG_IMAGE),
                    _ => (),
                }

                self.content_name = name;
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let column_slider = Slider::new(
            &mut self.column_slider,
            1.0..=16.0,
            16.0f32,
            |n| Message::ColumnCount(n.floor() as usize)
        )
            .width(Length::Units(64));

        let light_radio = Radio::new(
            Theme::Light,
            "Light",
            Some(self.hexview_theme),
            Message::ThemeSelected
        );
        let dark_radio = Radio::new(
            Theme::Dark,
            "Dark",
            Some(self.hexview_theme),
            Message::ThemeSelected
        );

        let content_list = PickList::new(
            &mut self.content_list,
            SAMPLE_OPTIONS,
            Some(self.content_name),
            Message::ContentSelected,
        );

        let row = Row::with_children(vec![
            Text::new("Column Count:").into(),
            column_slider.into(),
            Text::new("Theme:").into(),
            light_radio.into(),
            dark_radio.into(),
            Text::new("Content:").into(),
            content_list.into(),
        ])
            .align_items(Align::Center)
            .spacing(12)
            .padding(8);

        let hexview_theme: Box<dyn hexview_style::StyleSheet> = self.hexview_theme.into();
        let hexview = hexview::Hexview::new(&mut self.hexview)
            .style(hexview_theme);
        let scrollable = Scrollable::new(&mut self.scrollable)
            .push(hexview);

        let column = Column::with_children(vec![row.into(), scrollable.into()]);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Into<Box<dyn hexview_style::StyleSheet>> for Theme {
    fn into(self) -> Box<dyn hexview_style::StyleSheet> {
        match self {
            Theme::Light => Box::new(hexview_style::Light),
            Theme::Dark => Box::new(hexview_style::Dark),
        }
    }
}
