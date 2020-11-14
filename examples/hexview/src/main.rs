use iced::{
    radio, slider,
    Column, Container, Element, Length, Radio, Row, Sandbox,
    Settings, Slider, Text, VerticalAlignment,
};
use iced_widgets::{
    native::hexview,
    style::hexview as hexview_style,
};

#[derive(Debug, Clone)]
pub enum Message {
    ColumnCount(usize),
    ThemeSelected(Theme),
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
    hexview: hexview::State,
    column_slider: slider::State,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        App {
            hexview_theme: Theme::Light,
            hexview: hexview::State::new(),
            column_slider: slider::State::new(),
        }
    }

    fn title(&self) -> String {
        format!("manokara's iced widgets - hexview")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::ColumnCount(n) => self.hexview.set_column_count(n),
            Message::ThemeSelected(t) => self.hexview_theme = t,
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

        let row = Row::with_children(vec![
            Text::new("Column Count:").into(),
            column_slider.into(),
            Text::new("Theme:").into(),
            light_radio.into(),
            dark_radio.into(),
        ])
            .spacing(12)
            .padding(8);

        let hexview_theme: Box<dyn hexview_style::StyleSheet> = self.hexview_theme.into();
        let hexview = hexview::Hexview::new(&mut self.hexview)
            .style(hexview_theme);

        let column = Column::with_children(vec![row.into(), hexview.into()]);

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
