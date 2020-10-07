mod renderer;
mod ui;

use crate::renderer::Renderer;
use image::GenericImageView;
use sks::block::BackgroundType as SksBackgroundType;
use sks::block::Direction as SksDirection;
use std::collections::HashMap;
use std::convert::TryInto;

const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/bolonewt/bolonewt.ttf");
const M0_DATA: &[u8] = include_bytes!("../assets/images/M0.png");

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

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

#[derive(Debug)]
pub enum AppError {
    Render(crate::renderer::RenderError),
    Image(image::ImageError),
}

impl From<crate::renderer::RenderError> for AppError {
    fn from(e: crate::renderer::RenderError) -> AppError {
        AppError::Render(e)
    }
}

impl From<image::ImageError> for AppError {
    fn from(e: image::ImageError) -> Self {
        AppError::Image(e)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Render(e) => e.fmt(f),
            Self::Image(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for AppError {}

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
            self.insert(block, rendered.clone().into_bgra());
        }
    }
}

/// A Game Level
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Level {
    level_data: Vec<sks::Block>,
    is_dark: bool,
    background: SksBackgroundType,
}

impl Level {
    /// Create a new empty game level
    pub fn new() -> Self {
        Level {
            level_data: vec![sks::Block::Empty; sks::LEVEL_SIZE],
            is_dark: false,
            background: SksBackgroundType::Cobble,
        }
    }

    /// Get internal level data. This will not contain logic types like backgrounds and dark blocks
    pub fn get_level_data(&self) -> &[sks::Block] {
        &self.level_data
    }

    /// Try to insert a block at the index. Returns the block if it fails.
    pub fn add_block(&mut self, i: usize, block: sks::Block) -> Option<sks::Block> {
        if let Some(level_block) = self.level_data.get_mut(i) {
            *level_block = block;
            None
        } else {
            Some(block)
        }
    }

    /// Tries to import a level from a block array
    pub fn from_block_array(blocks: &[sks::Block]) -> Option<Self> {
        if blocks.len() != sks::LEVEL_SIZE {
            return None;
        }

        let mut level = Level::new();

        for (level_block, block) in level.level_data.iter_mut().zip(blocks.iter()) {
            let block = match block {
                sks::Block::Background { background_type } => {
                    level.background = background_type.clone();
                    sks::Block::Empty
                }
                sks::Block::Dark => {
                    level.is_dark = true;
                    sks::Block::Empty
                }
                b => b.clone(),
            };

            *level_block = block;
        }

        Some(level)
    }

    /// Tries to export a block array
    pub fn export_block_array(&self) -> Option<Vec<sks::Block>> {
        let mut to_insert = Vec::with_capacity(2);
        if self.is_dark() {
            to_insert.push(sks::Block::Dark);
        }

        if self.background != SksBackgroundType::Cobble {
            to_insert.push(sks::Block::Background {
                background_type: self.background.clone(),
            });
        }

        let data = self
            .get_level_data()
            .iter()
            .map(|block| {
                if block.is_empty() {
                    if let Some(block) = to_insert.pop() {
                        return block;
                    }
                }
                block.clone()
            })
            .collect();

        if to_insert.is_empty() {
            Some(data)
        } else {
            None
        }
    }

    /// Sets whether the level is dark
    pub fn set_dark(&mut self, is_dark: bool) {
        self.is_dark = is_dark;
    }

    /// Checks whether the level is dark
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}

pub struct App {
    renderer: Renderer,

    sks_image_renderer: sks::render::ImageRenderer,

    iced_state: iced_native::program::State<crate::ui::UiApp>,
    iced_debug: iced_native::Debug,
    iced_viewport: iced_wgpu::Viewport,
    iced_cursor_position: iced_core::Point,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        human_panic::setup_panic!();

        let mut renderer = futures::executor::block_on(Renderer::new())?;
        dbg!("{:#?}", renderer.wgpu_adapter.get_info());

        let mut sks_image_renderer = sks::render::ImageRenderer::new();

        let block_width = WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;
        let block_height = WINDOW_HEIGHT / sks::LEVEL_HEIGHT as u32;

        let invalid_block_image = image::RgbaImage::from_fn(block_width, block_height, |x, y| {
            let x = i64::from(x);
            let y = i64::from(y);
            let top = i64::from(block_height - 1);
            let right = i64::from(block_width - 1);
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
        let invalid_block_image = image::DynamicImage::ImageRgba8(invalid_block_image);

        let mut iced_block_map = IcedBlockMap::new(iced_native::image::Handle::from_pixels(
            invalid_block_image.width(),
            invalid_block_image.height(),
            invalid_block_image.into_bgra().into_vec(),
        ));

        let iced_background_image = iced_native::image::Handle::from_memory(M0_DATA.into());

        for block in BLOCKS.iter().cloned() {
            iced_block_map.generate(&mut sks_image_renderer, block);
        }

        let mut iced_debug = iced_native::Debug::new();
        let iced_viewport_size = iced_core::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let iced_viewport = iced_wgpu::Viewport::with_physical_size(iced_viewport_size, 1.0);
        let iced_app = crate::ui::UiApp::new(iced_block_map, iced_background_image);
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
        })
    }

    pub fn update(&mut self) {
        if !self.iced_state.is_queue_empty() {
            let _ = self.iced_state.update(
                self.iced_viewport.logical_size(),
                self.iced_cursor_position,
                None,
                &mut self.renderer.iced_renderer,
                &mut self.iced_debug,
            );
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
    pub fn get_active_block(&self) -> Option<&sks::Block> {
        self.iced_state.program().active_block.as_ref()
    }

    pub fn set_active_block(&mut self, block: Option<sks::Block>) {
        self.iced_state
            .queue_message(crate::ui::Message::ChangeActiveBlock { block });
    }

    pub fn get_level(&self) -> &Level {
        &self.iced_state.program().level
    }

    pub fn get_level_data(&self) -> &[sks::Block] {
        self.get_level().get_level_data()
    }

    pub fn add_block(&mut self, i: usize, block: sks::Block) {
        self.iced_state
            .queue_message(crate::ui::Message::AddBlock { index: i, block });
    }

    pub fn get_level_image(&mut self) -> Result<image::DynamicImage, sks::render::RenderError> {
        let opts = sks::render::RenderOptions {
            width: WINDOW_WIDTH.try_into().unwrap(),
            height: WINDOW_HEIGHT.try_into().unwrap(),
        };

        self.sks_image_renderer
            .render(self.iced_state.program().level.get_level_data(), &opts)
    }

    pub fn import(&mut self, blocks: &[sks::Block]) -> Option<()> {
        let level = Level::from_block_array(blocks)?;
        self.iced_state
            .queue_message(crate::ui::Message::ImportLevel { level });
        Some(())
    }

    pub fn set_dark(&mut self, dark: bool) {
        self.iced_state
            .queue_message(crate::ui::Message::SetDark { dark });
    }

    pub fn is_dark(&self) -> bool {
        self.get_level().is_dark()
    }

    pub fn export(&self) -> Option<Vec<sks::Block>> {
        self.get_level().export_block_array()
    }

    pub fn set_grid(&mut self, grid: bool) {
        self.iced_state
            .queue_message(crate::ui::Message::SetGrid { grid });
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
}
