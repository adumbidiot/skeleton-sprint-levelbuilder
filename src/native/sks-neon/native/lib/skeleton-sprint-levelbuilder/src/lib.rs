mod renderer;
mod ui;

use crate::renderer::Renderer;
use conrod_core::text::Font;
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

fn load_raw_image_to_conrod_image(
    renderer: &Renderer,
    image: &[u8],
) -> Result<conrod_wgpu::Image, AppError> {
    let rgba_image = image::load_from_memory(image)?;
    Ok(load_image_to_conrod_image(renderer, rgba_image.into_rgba()))
}

fn load_image_to_conrod_image(
    renderer: &Renderer,
    rgba_image: image::RgbaImage,
) -> conrod_wgpu::Image {
    let dimensions = rgba_image.dimensions();
    let texture = renderer.create_texture(rgba_image);

    conrod_wgpu::Image {
        texture,
        texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        width: dimensions.0,
        height: dimensions.1,
    }
}

pub struct ConrodBlockMap {
    map: HashMap<sks::block::Block, conrod_core::image::Id>,
    invalid: conrod_core::image::Id,
}

impl ConrodBlockMap {
    pub fn new(invalid: conrod_core::image::Id) -> Self {
        ConrodBlockMap {
            map: HashMap::new(),
            invalid,
        }
    }

    /// Note blocks have their data stripped on insert.
    pub fn insert(
        &mut self,
        renderer: &Renderer,
        image_map: &mut conrod_core::image::Map<conrod_wgpu::Image>,
        mut block: sks::Block,
        img: image::RgbaImage,
    ) {
        if let sks::block::Block::Note { text } = &mut block {
            text.clear();
        }

        let img = load_image_to_conrod_image(renderer, img);
        let img = image_map.insert(img);

        self.map.insert(block, img);
    }

    pub fn get(&self, mut block: sks::Block) -> conrod_core::image::Id {
        if let sks::block::Block::Note { text } = &mut block {
            text.clear();
        }

        *self.map.get(&block).unwrap_or(&self.invalid)
    }

    pub fn generate(
        &mut self,
        renderer: &Renderer,
        image_map: &mut conrod_core::image::Map<conrod_wgpu::Image>,
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
            self.insert(&renderer, image_map, block, rendered.clone().into_rgba());
        }
    }
}

/// A Game Level
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
    ui: conrod_core::Ui,
    ids: ui::Ids,
    font: conrod_core::text::font::Id,
    image_map: conrod_core::image::Map<conrod_wgpu::Image>,

    background_image: conrod_core::image::Id,
    invalid_block_image: conrod_core::image::Id,

    conrod_block_map: ConrodBlockMap,

    sks_image_renderer: sks::render::ImageRenderer,

    level: Level,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        human_panic::setup_panic!();

        let renderer = futures::executor::block_on(Renderer::new())?;
        dbg!("{:#?}", renderer.wgpu_adapter.get_info());

        let mut sks_image_renderer = sks::render::ImageRenderer::new();

        let font = Font::from_bytes(FONT_DATA).unwrap();
        let ui_size = [WINDOW_WIDTH.into(), WINDOW_HEIGHT.into()];

        let mut ui = conrod_core::UiBuilder::new(ui_size).build();
        let ids = ui::Ids::new(ui.widget_id_generator());
        let font = ui.fonts.insert(font);

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

        let mut image_map = conrod_core::image::Map::<conrod_wgpu::Image>::new();
        
        let invalid_block_image =
            load_image_to_conrod_image(&renderer, invalid_block_image.into_rgba());
        let invalid_block_image = image_map.insert(invalid_block_image);
        
        let background_image = load_raw_image_to_conrod_image(&renderer, M0_DATA).unwrap();
        let background_image = image_map.insert(background_image);

        let mut conrod_block_map = ConrodBlockMap::new(invalid_block_image);

        for block in BLOCKS.iter().cloned() {
            conrod_block_map.generate(&renderer, &mut image_map, &mut sks_image_renderer, block);
        }
        
        let level = Level::new();

        Ok(App {
            renderer,
            ui,
            ids,
            font,
            image_map,

            background_image,
            invalid_block_image,

            conrod_block_map,

            sks_image_renderer,

            level,
        })
    }

    pub fn update(&mut self) {
        let ui_data = ui::UiData {
            invalid_block_image: self.invalid_block_image,
            background_image: self.background_image,
            conrod_block_map: &self.conrod_block_map,
        };

        ui::gui(
            &mut self.ui.set_widgets(),
            &mut self.ids,
            &self.font,
            self.level.get_level_data(),
            ui_data,
        );
    }

    pub fn draw(&mut self) -> Result<(), AppError> {
        self.renderer.draw_conrod(&self.ui, &self.image_map)?;
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
    pub fn get_level_data(&self) -> &[sks::Block] {
        &self.level.get_level_data()
    }

    pub fn add_block(&mut self, i: usize, block: sks::Block) {
        let success = self.level.add_block(i, block).is_none();
        assert!(success);
    }

    pub fn get_level_image(&mut self) -> Result<image::DynamicImage, sks::render::RenderError> {
        let opts = sks::render::RenderOptions {
            width: WINDOW_WIDTH.try_into().unwrap(),
            height: WINDOW_HEIGHT.try_into().unwrap(),
        };

        self.sks_image_renderer
            .render(self.level.get_level_data(), &opts)
    }

    pub fn import(&mut self, blocks: &[sks::Block]) {
        self.level = Level::from_block_array(blocks).unwrap();
    }

    pub fn set_dark(&mut self, val: bool) {
        self.level.set_dark(val);
    }

    pub fn is_dark(&self) -> bool {
        self.level.is_dark()
    }

    pub fn export(&self) -> Option<Vec<sks::Block>> {
        self.level.export_block_array()
    }
}
