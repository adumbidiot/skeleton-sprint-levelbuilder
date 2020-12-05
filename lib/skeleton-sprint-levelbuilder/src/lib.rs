mod renderer;
pub mod ui;

use crate::renderer::Renderer;
use iced_core::keyboard::{
    KeyCode,
    Modifiers,
};
use image::GenericImageView;
pub use image::{
    DynamicImage,
    RgbaImage,
};
pub use sks;
use sks::{
    block::Direction as SksDirection,
    format::LevelNumber,
};
use std::{
    collections::HashMap,
    convert::TryInto,
};

pub const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/bolonewt/bolonewt.ttf");
const M0_DATA: &[u8] = include_bytes!("../assets/images/M0.png");
const TRASH_BIN_DATA: &[u8] = include_bytes!("../assets/images/trash-bin.png");

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;

const BLOCKS: &[sks::Block] = &[
    sks::Block::Block,
    sks::Block::Empty,
    sks::Block::Exit,
    sks::Block::Key,
    sks::Block::Lock,
    sks::Block::Note {
        text: String::new(),
    },
    sks::Block::Scaffold,
    sks::Block::SecretExit,
    sks::Block::Switch,
    sks::Block::SwitchCeiling,
    sks::Block::PipeIn,
    sks::Block::PipeOut,
    sks::Block::OneWayWall {
        direction: SksDirection::Up,
    },
    sks::Block::OneWayWall {
        direction: SksDirection::Down,
    },
    sks::Block::OneWayWall {
        direction: SksDirection::Left,
    },
    sks::Block::OneWayWall {
        direction: SksDirection::Right,
    },
    sks::Block::PipePhase,
    sks::Block::PipeSolid,
    sks::Block::Player,
    sks::Block::PowerUpBurrow,
    sks::Block::PowerUpRecall,
    sks::Block::ToggleBlock { solid: true },
    sks::Block::ToggleBlock { solid: false },
    sks::Block::Torch,
    sks::Block::Wire,
];

type BgraImage = image::ImageBuffer<image::Bgra<u8>, Vec<u8>>;

/// App error
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Render Error
    #[error("{0}")]
    Render(#[from] crate::renderer::RenderError),

    /// Image error
    #[error("{0}")]
    Image(#[from] image::ImageError),

    /// Native file dialog error
    #[error("{0}")]
    Nfd(#[from] win_nfd::NfdError),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Level Decode Error
    #[error("{0}")]
    SksDecode(#[from] sks::format::DecodeError),

    /// Tokio Join Error
    #[error("{0}")]
    JoinError(#[from] tokio::task::JoinError),
}

pub struct IcedBlockMap {
    map: HashMap<sks::block::Block, iced_native::image::Handle>,
    invalid: iced_native::image::Handle,
}

impl IcedBlockMap {
    pub fn new(invalid: iced_native::image::Handle) -> Self {
        Self {
            map: HashMap::new(),
            invalid,
        }
    }

    /// Note blocks have their data stripped on insert.
    pub fn insert(&mut self, mut block: sks::Block, img: BgraImage) {
        if let sks::Block::Note { text } = &mut block {
            text.clear();
        }

        let img =
            iced_native::image::Handle::from_pixels(img.width(), img.height(), img.into_vec());

        self.map.insert(block, img);
    }

    pub fn get(&self, mut block: sks::Block) -> iced_native::image::Handle {
        if let sks::Block::Note { text } = &mut block {
            text.clear();
        }

        self.map.get(&block).unwrap_or(&self.invalid).clone()
    }

    pub fn generate(
        &mut self,
        sks_image_renderer: &mut sks::render::ImageRenderer,
        block: sks::Block,
    ) {
        let block_width = WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;
        let block_height = WINDOW_HEIGHT / sks::LEVEL_HEIGHT as u32;

        let rendered = sks_image_renderer.get_rendered(sks::render::ImageRequest {
            w: block_width,
            h: block_height,
            block: block.clone(),
        });

        if let Some(rendered) = rendered {
            self.insert(block, rendered.clone().into_bgra8());
        }
    }
}

pub fn render_invalid_block_image(block_size: u32) -> image::DynamicImage {
    let invalid_block_image = image::RgbaImage::from_fn(block_size, block_size, |x, y| {
        let x = i64::from(x);
        let y = i64::from(y);
        let top = i64::from(block_size - 1);
        let right = i64::from(block_size - 1);
        let limit = 4;

        if (x - y).abs() < limit
            || (x - (top - y)).abs() < limit
            || (x - right).abs() < limit
            || (y - top).abs() < limit
            || x < limit
            || y < limit
        {
            image::Rgba([0, 0, 0, 255])
        } else {
            image::Rgba([255, 105, 180, 255])
        }
    });

    image::DynamicImage::ImageRgba8(invalid_block_image)
}

pub fn init_iced_block_map(
    invalid_block_image: image::DynamicImage,
    sks_image_renderer: &mut sks::render::ImageRenderer,
) -> IcedBlockMap {
    let invalid_block_image = iced::image::Handle::from_pixels(
        invalid_block_image.width(),
        invalid_block_image.height(),
        invalid_block_image.into_bgra8().into_vec(),
    );
    let mut iced_block_map = IcedBlockMap::new(invalid_block_image);

    for block in BLOCKS.iter().cloned() {
        iced_block_map.generate(sks_image_renderer, block);
    }

    iced_block_map
}

pub struct App {
    renderer: Renderer,

    sks_image_renderer: sks::render::ImageRenderer,

    pub iced_state: iced_native::program::State<crate::ui::UiApp>,
    iced_debug: iced_native::Debug,
    iced_viewport: iced_wgpu::Viewport,
    iced_cursor_position: iced_core::Point,
    iced_modifiers: Modifiers,

    tokio_rt: tokio::runtime::Runtime,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        human_panic::setup_panic!();

        let mut renderer = futures::executor::block_on(Renderer::new())?;
        dbg!(renderer.wgpu_adapter.get_info());

        let mut sks_image_renderer = sks::render::ImageRenderer::new();

        let block_size = WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;
        let invalid_block_image = render_invalid_block_image(block_size);

        let iced_block_map = init_iced_block_map(invalid_block_image, &mut sks_image_renderer);

        let iced_app = self::ui::UiApp::new(iced_block_map);

        let mut iced_debug = iced_native::Debug::new();
        let iced_viewport_size = iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let iced_viewport = iced_wgpu::Viewport::with_physical_size(iced_viewport_size, 1.0);
        let iced_cursor_position = iced_core::Point::new(0.0, 0.0);
        let iced_state = iced_native::program::State::new(
            iced_app,
            iced_viewport.logical_size(),
            iced_cursor_position,
            &mut renderer.iced_renderer,
            &mut iced_debug,
        );

        Ok(App {
            renderer,

            sks_image_renderer,

            iced_state,
            iced_debug,
            iced_viewport,
            iced_cursor_position,
            iced_modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                logo: false,
            },

            tokio_rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?,
        })
    }

    pub fn update(&mut self) {
        if !self.iced_state.is_queue_empty() {
            let cmd = self.iced_state.update(
                self.iced_viewport.logical_size(),
                self.iced_cursor_position,
                None,
                &mut self.renderer.iced_renderer,
                &mut self.iced_debug,
            );

            for msg in self.tokio_rt.block_on(futures::future::join_all(
                cmd.unwrap_or_else(iced::Command::none).futures(),
            )) {
                self.iced_state.queue_message(msg);
            }
        }
    }

    pub fn draw(&mut self) -> Result<(), AppError> {
        self.renderer.draw_ui(&self.iced_state, &self.iced_debug)?;
        Ok(())
    }

    pub fn get_raw_image(&mut self) -> Result<image::RgbaImage, AppError> {
        self.update();
        self.draw()?;
        let data = self.renderer.get_output_rgba_image()?;

        Ok(data)
    }
}

/// Intended to be temp interface
impl App {
    pub fn get_level_number(&self) -> Option<LevelNumber> {
        self.get_level().get_level_number().cloned()
    }

    pub fn set_level_number(&mut self, n: LevelNumber) {
        self.iced_state
            .queue_message(crate::ui::Message::SetLevelNumber(Some(n)));
    }

    pub fn get_level(&self) -> &sks::Level {
        &self.iced_state.program().level
    }

    pub fn get_level_data(&self) -> &[sks::Block] {
        self.get_level().get_level_data()
    }

    pub fn get_level_image(&mut self) -> Result<image::DynamicImage, sks::render::RenderError> {
        let opts = sks::render::RenderOptions {
            width: WINDOW_WIDTH.try_into().unwrap(),
            height: WINDOW_HEIGHT.try_into().unwrap(),
        };

        self.sks_image_renderer
            .render(self.iced_state.program().level.get_level_data(), &opts)
    }

    pub fn export(&self) -> Option<Vec<sks::Block>> {
        self.get_level().export_block_array()
    }

    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.iced_cursor_position = iced_core::Point::new(x as f32, y as f32);
        let event = iced_native::Event::Mouse(iced_native::mouse::Event::CursorMoved {
            x: x as f32,
            y: y as f32,
        });
        self.iced_state.queue_event(event);
    }

    pub fn emit_left_mouse_button_down(&mut self) {
        let event = iced_native::Event::Mouse(iced_native::mouse::Event::ButtonPressed(
            iced_native::mouse::Button::Left,
        ));
        self.iced_state.queue_event(event);
    }

    pub fn emit_right_mouse_button_down(&mut self) {
        let event = iced_native::Event::Mouse(iced_native::mouse::Event::ButtonPressed(
            iced_native::mouse::Button::Right,
        ));
        self.iced_state.queue_event(event);
    }

    pub fn emit_left_mouse_button_up(&mut self) {
        let event = iced_native::Event::Mouse(iced_native::mouse::Event::ButtonReleased(
            iced_native::mouse::Button::Left,
        ));
        self.iced_state.queue_event(event);
    }

    pub fn emit_right_mouse_button_up(&mut self) {
        let event = iced_native::Event::Mouse(iced_native::mouse::Event::ButtonReleased(
            iced_native::mouse::Button::Right,
        ));
        self.iced_state.queue_event(event);
    }

    pub fn emit_keyboard_key_down(&mut self, key_code: u64) {
        if let Some(key_code) = translate_key_code(key_code) {
            let event = iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                key_code,
                modifiers: self.iced_modifiers,
            });
            self.iced_state.queue_event(event);
        }
    }

    pub fn emit_keyboard_key_up(&mut self, key_code: u64) {
        if let Some(key_code) = translate_key_code(key_code) {
            let event = iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyReleased {
                key_code,
                modifiers: self.iced_modifiers,
            });
            self.iced_state.queue_event(event);
        }
    }

    pub fn emit_recieved_char(&mut self, c: char) {
        let event =
            iced_native::Event::Keyboard(iced_native::keyboard::Event::CharacterReceived(c));
        self.iced_state.queue_event(event);
    }
}

fn translate_key_code(key_code: u64) -> Option<KeyCode> {
    match key_code {
        8 => Some(KeyCode::Backspace),
        13 => Some(KeyCode::Enter),
        37 => Some(KeyCode::Left),
        38 => Some(KeyCode::Up),
        39 => Some(KeyCode::Right),
        40 => Some(KeyCode::Down),
        46 => Some(KeyCode::Delete),
        65 => Some(KeyCode::A),
        68 => Some(KeyCode::D),
        83 => Some(KeyCode::S),
        87 => Some(KeyCode::W),
        90 => Some(KeyCode::Z),
        code => {
            eprintln!("Unknown key code: {}", code);
            None
        }
    }
}
