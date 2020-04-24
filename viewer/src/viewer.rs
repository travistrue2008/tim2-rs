use iced::{
    executor,
    widget::{button, image, image_pane, scrollable, text_input},
    Align, Application, Button, Column, Command, Container, Element, ImagePane, Length, Row,
    Scrollable, Subscription, Text, TextInput,
};
use iced_native::input::{
    keyboard::{self, KeyCode},
    mouse::{self, ScrollDelta},
    ButtonState,
};
use std::path::PathBuf;

pub struct Viewer {
    state: State,
    handle: Option<image::Handle>,
    image_pane_state: image_pane::State,
    image_title: String,
    error_msg: String,
    directory_tree: DirectoryTree,
    directory_search: DirectorySearch,
    ctrl_pressed: bool,
    scale: u16,
}

enum State {
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadDirectory(PathBuf),
    LoadedPaths((Vec<PathBuf>, Vec<PathBuf>)),
    NextFile,
    PrevFile,
    ChooseFile(usize),
    Search(String),
    HandleEvent(iced_native::Event),
    ScaleImage(f32),
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
                image_pane_state: image_pane::State::new(),
                image_title: String::new(),
                directory_tree: DirectoryTree::default(),
                directory_search: DirectorySearch::default(),
                ctrl_pressed: false,
                scale: 600,
            },
            Command::perform(async { flags.directory }, Message::LoadDirectory),
        )
    }

    fn title(&self) -> String {
        let title = match self.state {
            State::Loading => "Loading",
            _ => self.image_title.as_str(),
        };

        format!("Tim2 Viewer - {}", title)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LoadDirectory(directory) => {
                self.directory_tree.path = directory.clone();

                return Command::perform(load_directory(directory), Message::LoadedPaths);
            }
            Message::LoadedPaths((folders, files)) => {
                self.directory_tree.entries = files
                    .into_iter()
                    .enumerate()
                    .map(DirectoryEntry::from)
                    .collect();

                self.directory_tree.folders = folders
                    .into_iter()
                    .enumerate()
                    .map(DirectoryEntry::from)
                    .collect();

                self.directory_tree.idx = 0;
                self.directory_tree.query = String::new();
                self.directory_search.search = String::new();
                self.directory_tree.update_filter();

                if self.check_paths_exist() {
                    self.load_image();
                }
            }
            Message::NextFile => {
                if self.check_paths_exist() {
                    self.directory_tree.idx =
                        (self.directory_tree.idx + 1) % self.directory_tree.filtered_entries.len();

                    self.load_image();
                }
            }
            Message::PrevFile => {
                if self.check_paths_exist() {
                    self.directory_tree.idx = if self.directory_tree.idx == 0 {
                        self.directory_tree.filtered_entries.len() - 1
                    } else {
                        self.directory_tree.idx - 1
                    };

                    self.load_image();
                }
            }
            Message::ChooseFile(idx) => {
                if self.check_paths_exist() {
                    self.directory_tree.idx = idx;

                    self.load_image();
                }
            }
            Message::Search(search) => {
                self.directory_search.search = search.clone();
                self.directory_tree.query = search;
                self.directory_tree.update_filter();
            }
            Message::ScaleImage(scale) => {
                if scale > 0.0 && self.scale < 3000 {
                    self.scale += 30;
                } else if scale < 0.0 && self.scale > 30 {
                    self.scale -= 30;
                }
            }
            Message::HandleEvent(event) => match event {
                iced_native::Event::Keyboard(keyboard) => {
                    if let keyboard::Event::Input {
                        state, key_code, ..
                    } = keyboard
                    {
                        if state == ButtonState::Pressed {
                            match key_code {
                                KeyCode::Left => return self.update(Message::PrevFile),
                                KeyCode::Right => return self.update(Message::NextFile),
                                KeyCode::LControl | KeyCode::RControl => self.ctrl_pressed = true,
                                _ => {}
                            }
                        } else if key_code == KeyCode::LControl || key_code == KeyCode::RControl {
                            self.ctrl_pressed = false
                        }
                    }
                }
                iced_native::Event::Mouse(mouse) => {
                    if let mouse::Event::WheelScrolled { delta } = mouse {
                        if self.ctrl_pressed {
                            if let ScrollDelta::Lines { y, .. } = delta {
                                return self.update(Message::ScaleImage(y));
                            }
                        }
                    }
                }
                _ => {}
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::HandleEvent)
    }

    fn view(&mut self) -> Element<Self::Message> {
        Container::new(
            Row::new()
                .spacing(0)
                .push(
                    Container::new(
                        Column::new()
                            .spacing(15)
                            .push(
                                Container::new(self.directory_search.view())
                                    .width(Length::Fill)
                                    .align_x(Align::Start)
                                    .style(style::Theme),
                            )
                            .push(
                                Container::new(self.directory_tree.view())
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .align_x(Align::Start)
                                    .padding(3)
                                    .style(style::ImageContainer),
                            ),
                    )
                    .width(Length::Units(325))
                    .height(Length::Fill)
                    .align_x(Align::Start)
                    .padding(10)
                    .style(style::Theme),
                )
                .push(
                    Container::new(
                        Column::new().push(match self.state {
                            State::Loading => Container::new(Text::new("Loading..."))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x()
                                .center_y()
                                .style(style::ImageContainer),

                            State::Loaded => Container::new(
                                ImagePane::new(
                                    &mut self.image_pane_state,
                                    self.handle.as_ref().unwrap().clone(),
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .padding(5),
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x()
                            .center_y()
                            .style(style::ImageContainer),

                            State::Error => {
                                Container::new(Text::new(format!("ERROR: {}", self.error_msg)))
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .center_x()
                                    .center_y()
                                    .style(style::ImageContainer)
                            }
                        }),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .align_x(Align::Start)
                    .padding(10)
                    .style(style::Theme),
                ),
        )
        .style(style::MainContainer)
        .into()
    }
}

impl Viewer {
    fn load_image(&mut self) {
        let entry = &self.directory_tree.filtered_entries[self.directory_tree.idx];

        self.image_title = entry
            .path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned();

        let load_result = std::panic::catch_unwind(|| tim2::load(&entry.path).unwrap());

        match load_result {
            Ok(tim2) => {
                let frame = tim2.get_frame(0);
                let pixels = frame.to_raw(None);

                self.handle = Some(image::Handle::from_pixels(
                    frame.width() as _,
                    frame.height() as _,
                    pixels,
                ));

                //self.image_pane_state = image_pane::State::new();

                self.state = State::Loaded;
            }
            Err(_) => {
                self.error_msg = "Failed to load image ".to_owned();

                self.state = State::Error;
            }
        }
    }

    fn check_paths_exist(&mut self) -> bool {
        if self.directory_tree.filtered_entries.is_empty() {
            self.error_msg = "No .tm2 files found, try a different directory".to_owned();

            self.state = State::Error;

            self.image_title = "".to_owned();

            return false;
        }

        true
    }
}

async fn load_directory(directory: PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut folders = vec![];
    let mut files = vec![];

    if let Ok(dir_iter) = std::fs::read_dir(directory) {
        for entry_maybe in dir_iter {
            if let Ok(entry) = entry_maybe {
                let path = entry.path();

                if path.is_dir() {
                    folders.push(path);
                } else if let Some(ext) = path.extension() {
                    if ext.to_str().unwrap_or_default() == "tm2" {
                        files.push(path);
                    }
                }
            }
        }
    };

    folders.sort_by_key(|e| {
        e.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned()
    });

    files.sort_by_key(|e| {
        e.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned()
    });

    (folders, files)
}

#[derive(Default)]
struct DirectoryTree {
    path: PathBuf,
    state: scrollable::State,
    button_state: button::State,
    folders: Vec<DirectoryEntry>,
    filtered_folders: Vec<DirectoryEntry>,
    entries: Vec<DirectoryEntry>,
    filtered_entries: Vec<DirectoryEntry>,
    idx: usize,
    pub query: String,
}

impl DirectoryTree {
    fn view<'a>(&'a mut self) -> Element<Message> {
        let mut scroll = Scrollable::new(&mut self.state)
            .style(style::Theme)
            .width(Length::Fill);

        let button: Element<'a, Message> = Container::new(
            Button::new(&mut self.button_state, Text::new(".."))
                .width(Length::Units(283))
                .style(style::Theme)
                .on_press({
                    let current_path = self.path.clone();

                    let parent_dir = if let Some(path) = current_path.parent() {
                        path.to_owned()
                    } else {
                        current_path
                    };

                    Message::LoadDirectory(parent_dir)
                }),
        )
        .width(Length::Fill)
        .style(style::ScrollableItem)
        .into();

        scroll = scroll.push(button);

        for (idx, entry) in self.filtered_entries.iter_mut().enumerate() {
            let button: Element<'a, Message> = Container::new(
                Button::new(
                    &mut entry.state,
                    Text::new(
                        entry
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_owned(),
                    ),
                )
                .width(Length::Units(283))
                .style(style::Theme)
                .on_press(Message::ChooseFile(idx)),
            )
            .width(Length::Fill)
            .style(style::ScrollableItem)
            .into();

            scroll = scroll.push(button);
        }

        for entry in self.filtered_folders.iter_mut() {
            let button: Element<'a, Message> = Container::new(
                Button::new(
                    &mut entry.state,
                    Text::new(format!(
                        "{}/",
                        entry
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                    )),
                )
                .width(Length::Units(283))
                .style(style::Theme)
                .on_press(Message::LoadDirectory(entry.path.clone())),
            )
            .width(Length::Fill)
            .style(style::ScrollableItem)
            .into();

            scroll = scroll.push(button);
        }

        scroll.into()
    }

    fn update_filter(&mut self) {
        self.filtered_entries = self
            .entries
            .iter()
            .cloned()
            .filter(|entry| {
                let entry_path = entry.path.clone();
                let entry_name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_lowercase();

                entry_name.contains(&self.query.to_lowercase())
            })
            .collect();

        self.filtered_folders = self
            .folders
            .iter()
            .cloned()
            .filter(|entry| {
                let entry_path = entry.path.clone();
                let entry_name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_lowercase();

                entry_name.contains(&self.query.to_lowercase())
            })
            .collect();
    }
}

#[derive(Clone)]
struct DirectoryEntry {
    pub idx: usize,
    pub state: button::State,
    pub path: PathBuf,
}

impl From<(usize, PathBuf)> for DirectoryEntry {
    fn from(args: (usize, PathBuf)) -> Self {
        DirectoryEntry {
            idx: args.0,
            state: button::State::new(),
            path: args.1,
        }
    }
}

#[derive(Default)]
struct DirectorySearch {
    pub state: text_input::State,
    pub search: String,
}

impl DirectorySearch {
    fn view(&mut self) -> Element<Message> {
        TextInput::new(&mut self.state, "Search...", &self.search, |string| {
            Message::Search(string)
        })
        .width(Length::Fill)
        .size(30)
        .padding(2)
        .style(style::Theme)
        .into()
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
        0x1d as f32 / 255.0,
        0x1d as f32 / 255.0,
        0x1d as f32 / 255.0,
    );

    const ACCENT: Color = Color::from_rgb(
        0x4F as f32 / 255.0,
        0xa2 as f32 / 255.0,
        0xe1 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x4F as f32 / 255.0,
        0xa2 as f32 / 255.0,
        0xe1 as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x4F as f32 / 255.0,
        0xa2 as f32 / 255.0,
        0xe1 as f32 / 255.0,
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
                background: Some(Background::Color(Color::from_rgb8(0x2C, 0x2C, 0x2C))),
                text_color: Some(Color::WHITE),
                border_radius: 3,
                ..container::Style::default()
            }
        }
    }

    pub struct MainContainer;

    impl container::StyleSheet for MainContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2C, 0x2C, 0x2C))),
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            }
        }
    }

    pub struct ImageContainer;

    impl container::StyleSheet for ImageContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(SURFACE)),
                text_color: Some(Color::WHITE),
                border_radius: 3,
                ..container::Style::default()
            }
        }
    }

    pub struct ScrollableItem;

    impl container::StyleSheet for ScrollableItem {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(SURFACE)),
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
                border_radius: 3,
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
                background: Some(Background::Color(SURFACE)),
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

    pub struct FolderButton;

    impl button::StyleSheet for FolderButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x4e, 0x4e, 0x4e))),
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
                border_radius: 3,
                border_width: 0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: ACTIVE,
                    border_radius: 3,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        fn hovered(&self) -> scrollable::Scrollbar {
            let active = self.active();

            scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgba8(0x2c, 0x2c, 0x2c, 0.5))),
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
