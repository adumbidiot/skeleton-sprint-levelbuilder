mod render;

pub use sks::block::BackgroundType;
pub use sks::block::Block;
pub use sks::block::Direction;
pub use sks::format::as3::LevelNum;
pub use sks::format::decode;
pub use sks::format::FileFormat;
pub use image::DynamicImage;
pub use image::RgbaImage;

pub struct LevelBuilder {
    level_num: LevelNum,
    
    pub skeleton_sprint_levelbuilder: skeleton_sprint_levelbuilder::App,
}

impl LevelBuilder {
    pub fn new() -> Self {
        LevelBuilder {
            level_num: LevelNum::String('x'.to_string()),
            
            skeleton_sprint_levelbuilder: skeleton_sprint_levelbuilder::App::new().expect("LevelBuilder Init"),
        }
    }
    
    pub fn set_grid(&mut self, grid: bool) {
        self.skeleton_sprint_levelbuilder.set_grid(grid);
    }

    pub fn set_dark(&mut self, val: bool) {
        self.skeleton_sprint_levelbuilder.set_dark(val);
    }

    pub fn get_dark(&self) -> bool {
        self.skeleton_sprint_levelbuilder.is_dark()
    }

    pub fn add_block(&mut self, i: usize, block: Block) {        
        self.skeleton_sprint_levelbuilder.add_block(i, block);
    }

    pub fn get_level_data(&self) -> &[Block] {
        self.skeleton_sprint_levelbuilder.get_level_data()
    }

    pub fn render_image(&mut self) -> Result<image::DynamicImage, sks::render::RenderError> {
        self.skeleton_sprint_levelbuilder.get_level_image()
    }

    pub fn export(&self) -> Option<Vec<Block>> {
        self.skeleton_sprint_levelbuilder.export()
    }

    pub fn export_format(&self, format: &FileFormat) -> Option<String> {
        sks::format::encode(&self.export()?, format, Some(&self.level_num)).ok()
    }

    /// Imports a block array
    pub fn import(&mut self, level: &[Block]) {
         self.skeleton_sprint_levelbuilder.import(&level);
    }

    pub fn set_level(&mut self, level_num: LevelNum) {
        self.level_num = level_num;
    }
}
