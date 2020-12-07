#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::Application;
use iced_native::program::Program;

pub struct AppWrap(skeleton_sprint_levelbuilder::ui::UiApp);

impl AppWrap {
    pub fn new() -> Self {
        let mut sks_image_renderer = sks::render::ImageRenderer::new();
        let block_size = skeleton_sprint_levelbuilder::WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;
        let invalid_block_image =
            skeleton_sprint_levelbuilder::render_invalid_block_image(block_size);
        let iced_block_map = skeleton_sprint_levelbuilder::init_iced_block_map(
            invalid_block_image,
            &mut sks_image_renderer,
        );

        AppWrap(skeleton_sprint_levelbuilder::ui::UiApp::new(iced_block_map))
    }
}

impl Default for AppWrap {
    fn default() -> Self {
        Self::new()
    }
}

// This is nothing but a test area for now
impl Application for AppWrap {
    type Executor = iced::executor::Default;
    type Message = skeleton_sprint_levelbuilder::ui::Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (Self::new(), iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Skeleton Sprint Levelbuilder")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        self.0.update(message)
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        self.0.view()
    }

    fn background_color(&self) -> iced::Color {
        iced::Color::BLACK
    }
}

fn main() {
    AppWrap::run(iced::Settings {
        default_font: Some(skeleton_sprint_levelbuilder::FONT_DATA),
        window: iced::window::Settings {
            size: (1280, 720),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
}
