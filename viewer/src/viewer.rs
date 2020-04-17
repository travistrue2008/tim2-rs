use iced::{
    executor,
    widget::{image, scrollable},
    Align, Application, Column, Command, Container, Element, Length, Row, Scrollable, Subscription,
    Text,
};
use iced_native::input::{
    keyboard::{self, KeyCode},
    ButtonState,
};
use std::path::PathBuf;

pub struct Viewer {
    state: State,
    handle: Option<image::Handle>,
    error_msg: String,
    directory_tree: DirectoryTree,
}

enum State {
    Loading,
    Loaded,
    Error,
}

#[derive(Debug)]
pub enum Message {
    LoadedPaths(Vec<PathBuf>),
    NextFile,
    PrevFile,
    HandleEvent(iced_native::Event),
}

#[derive(Default)]
pub struct Flags {
    pub directory: PathBuf,
}

impl Application for Viewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Viewer {
                state: State::Loading,
                handle: None,
                error_msg: String::new(),
                directory_tree: DirectoryTree::default(),
            },
            Command::perform(load_paths(flags.directory), Message::LoadedPaths),
        )
    }

    fn title(&self) -> String {
        let title = match self.state {
            State::Loading => "Loading",
            _ => {
                if self.directory_tree.paths.is_empty() {
                    ""
                } else {
                    let path = &self.directory_tree.paths[self.directory_tree.path_idx];

                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                }
            }
        };

        format!("Tim2 Viewer - {}", title)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LoadedPaths(paths) => {
                self.directory_tree.paths = paths;

                if self.check_paths_exist() {
                    self.load_image();
                }
            }
            Message::NextFile => {
                if self.check_paths_exist() {
                    self.directory_tree.path_idx =
                        (self.directory_tree.path_idx + 1) % self.directory_tree.paths.len();

                    self.load_image();
                }
            }
            Message::PrevFile => {
                if self.check_paths_exist() {
                    self.directory_tree.path_idx = if self.directory_tree.path_idx == 0 {
                        self.directory_tree.paths.len() - 1
                    } else {
                        self.directory_tree.path_idx - 1
                    };

                    self.load_image();
                }
            }
            Message::HandleEvent(event) => {
                if let iced_native::Event::Keyboard(keyboard) = event {
                    if let keyboard::Event::Input {
                        state, key_code, ..
                    } = keyboard
                    {
                        if state == ButtonState::Pressed {
                            match key_code {
                                KeyCode::Left => return self.update(Message::PrevFile),
                                KeyCode::Right => return self.update(Message::NextFile),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::HandleEvent)
    }

    fn view(&mut self) -> Element<Self::Message> {
        Row::new()
            .spacing(0)
            .push(
                Container::new(self.directory_tree.view())
                    .width(Length::Shrink)
                    .height(Length::Fill)
                    .align_x(Align::Start)
                    .padding(0)
                    .style(style::Theme),
            )
            .push(match self.state {
                State::Loading => Container::new(Text::new("Loading..."))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(0)
                    .center_x()
                    .center_y()
                    .style(style::Theme),

                State::Loaded => {
                    let image = image::Image::new(self.handle.as_ref().unwrap().clone());

                    Container::new(image)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(0)
                        .center_x()
                        .center_y()
                        .style(style::Theme)
                }

                State::Error => Container::new(Text::new(format!(
                    "ERROR: {}\n\nTry another image",
                    self.error_msg
                )))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .center_x()
                .center_y()
                .style(style::Theme),
            })
            .into()
    }
}

impl Viewer {
    fn load_image(&mut self) {
        let path = &self.directory_tree.paths[self.directory_tree.path_idx];

        let load_result = std::panic::catch_unwind(|| tim2::load(path).unwrap());

        match load_result {
            Ok(tim2) => {
                let frame = tim2.get_frame(0);
                let pixels = frame.to_raw(None);

                self.handle = Some(image::Handle::from_pixels(
                    frame.width() as _,
                    frame.height() as _,
                    pixels,
                ));

                self.state = State::Loaded;
            }
            Err(_) => {
                self.error_msg = "Failed to load image ".to_owned();

                self.state = State::Error;
            }
        }
    }

    fn check_paths_exist(&mut self) -> bool {
        if self.directory_tree.paths.is_empty() {
            self.error_msg = "No .tm2 files found, try a different directory".to_owned();

            self.state = State::Error;

            return false;
        }

        true
    }
}

async fn load_paths(directory: PathBuf) -> Vec<PathBuf> {
    let mut paths = vec![];

    let query = format!("{}/**/*.tm2", directory.display());

    if let Ok(glob) = glob::glob(&query) {
        for file in glob {
            if let Ok(path) = file {
                paths.push(path)
            }
        }
    }

    paths
}

#[derive(Default)]
struct DirectoryTree {
    state: scrollable::State,
    paths: Vec<PathBuf>,
    path_idx: usize,
}

impl DirectoryTree {
    fn view(&mut self) -> Element<Message> {
        let mut scrollable = Scrollable::new(&mut self.state).style(style::Theme);

        for path in self.paths.iter() {
            scrollable = scrollable.push(Text::new(
                path.file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            ));
        }

        scrollable.into()
    }
}

mod style {
    #![allow(clippy::unnecessary_cast)]

    use iced::{
        button, checkbox, container, progress_bar, radio, scrollable, slider, text_input,
        Background, Color,
    };

    pub struct Theme;

    const SURFACE: Color = Color::from_rgb(
        0x40 as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x4B as f32 / 255.0,
    );

    const ACCENT: Color = Color::from_rgb(
        0x6F as f32 / 255.0,
        0xFF as f32 / 255.0,
        0xE9 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x72 as f32 / 255.0,
        0x89 as f32 / 255.0,
        0xDA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    impl From<Theme> for Box<dyn container::StyleSheet> {
        fn from(_: Theme) -> Self {
            Container.into()
        }
    }

    impl From<Theme> for Box<dyn radio::StyleSheet> {
        fn from(_: Theme) -> Self {
            Radio.into()
        }
    }

    impl From<Theme> for Box<dyn text_input::StyleSheet> {
        fn from(_: Theme) -> Self {
            TextInput.into()
        }
    }

    impl From<Theme> for Box<dyn button::StyleSheet> {
        fn from(_: Theme) -> Self {
            Button.into()
        }
    }

    impl From<Theme> for Box<dyn scrollable::StyleSheet> {
        fn from(_: Theme) -> Self {
            Scrollable.into()
        }
    }

    impl From<Theme> for Box<dyn slider::StyleSheet> {
        fn from(_: Theme) -> Self {
            Slider.into()
        }
    }

    impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
        fn from(_: Theme) -> Self {
            ProgressBar.into()
        }
    }

    impl From<Theme> for Box<dyn checkbox::StyleSheet> {
        fn from(_: Theme) -> Self {
            Checkbox.into()
        }
    }

    struct Container;

    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            }
        }
    }

    struct Radio;

    impl radio::StyleSheet for Radio {
        fn active(&self) -> radio::Style {
            radio::Style {
                background: Background::Color(SURFACE),
                dot_color: ACTIVE,
                border_width: 1,
                border_color: ACTIVE,
            }
        }

        fn hovered(&self) -> radio::Style {
            radio::Style {
                background: Background::Color(Color { a: 0.5, ..SURFACE }),
                ..self.active()
            }
        }
    }

    struct TextInput;

    impl text_input::StyleSheet for TextInput {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(SURFACE),
                border_radius: 2,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1,
                border_color: ACCENT,
                ..self.active()
            }
        }

        fn hovered(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1,
                border_color: Color { a: 0.3, ..ACCENT },
                ..self.focused()
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgb(0.4, 0.4, 0.4)
        }

        fn value_color(&self) -> Color {
            Color::WHITE
        }

        fn selection_color(&self) -> Color {
            ACTIVE
        }
    }

    struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(ACTIVE)),
                border_radius: 3,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(HOVERED)),
                text_color: Color::WHITE,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1,
                border_color: Color::WHITE,
                ..self.hovered()
            }
        }
    }

    struct Scrollable;

    impl scrollable::StyleSheet for Scrollable {
        fn active(&self) -> scrollable::Scrollbar {
            scrollable::Scrollbar {
                background: Some(Background::Color(SURFACE)),
                border_radius: 2,
                border_width: 0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: ACTIVE,
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        fn hovered(&self) -> scrollable::Scrollbar {
            let active = self.active();

            scrollable::Scrollbar {
                background: Some(Background::Color(Color { a: 0.5, ..SURFACE })),
                scroller: scrollable::Scroller {
                    color: HOVERED,
                    ..active.scroller
                },
                ..active
            }
        }

        fn dragging(&self) -> scrollable::Scrollbar {
            let hovered = self.hovered();

            scrollable::Scrollbar {
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.85, 0.85, 0.85),
                    ..hovered.scroller
                },
                ..hovered
            }
        }
    }

    struct Slider;

    impl slider::StyleSheet for Slider {
        fn active(&self) -> slider::Style {
            slider::Style {
                rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 9 },
                    color: ACTIVE,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        fn hovered(&self) -> slider::Style {
            let active = self.active();

            slider::Style {
                handle: slider::Handle {
                    color: HOVERED,
                    ..active.handle
                },
                ..active
            }
        }

        fn dragging(&self) -> slider::Style {
            let active = self.active();

            slider::Style {
                handle: slider::Handle {
                    color: Color::from_rgb(0.85, 0.85, 0.85),
                    ..active.handle
                },
                ..active
            }
        }
    }

    struct ProgressBar;

    impl progress_bar::StyleSheet for ProgressBar {
        fn style(&self) -> progress_bar::Style {
            progress_bar::Style {
                background: Background::Color(SURFACE),
                bar: Background::Color(ACTIVE),
                border_radius: 10,
            }
        }
    }

    struct Checkbox;

    impl checkbox::StyleSheet for Checkbox {
        fn active(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(if is_checked { ACTIVE } else { SURFACE }),
                checkmark_color: Color::WHITE,
                border_radius: 2,
                border_width: 1,
                border_color: ACTIVE,
            }
        }

        fn hovered(&self, is_checked: bool) -> checkbox::Style {
            checkbox::Style {
                background: Background::Color(Color {
                    a: 0.8,
                    ..if is_checked { ACTIVE } else { SURFACE }
                }),
                ..self.active(is_checked)
            }
        }
    }
}
