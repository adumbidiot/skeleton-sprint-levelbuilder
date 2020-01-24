mod as3;

pub use as3::encode_as3;
pub use sks::block::BackgroundType;
pub use sks::block::Block;
pub use sks::block::Direction;

use sks::LEVEL_WIDTH;
use sks::LEVEL_HEIGHT;

///Legacy Compat
pub fn decode_lbl(data: &str) -> Option<Vec<Block>> {
    sks::format::lbl::decode(data).ok()
}

pub fn decode_any(data: &str) -> Option<Vec<Block>> {
	sks::format::decode(&data).ok()
}
///End Legacy Compat

pub const M0_BG: &[u8] = include_bytes!("../assets/M0.png");

/*
impl Block {
	pub fn as_id(&self) -> usize {
		match self {
            Block::Background {
                background_type: BackgroundType::Cobble,
            } => "M0".into(),
            Block::Background {
                background_type: BackgroundType::Waterfall,
            } => "M1".into(),
            Block::Background {
                background_type: BackgroundType::Skullfall,
            } => "M2".into(),
            Block::Background {
                background_type: BackgroundType::Concrete,
            } => "M3".into(),
            Block::Background {
                background_type: BackgroundType::Reserved1,
            } => "M4".into(),
            Block::Background {
                background_type: BackgroundType::Reserved2,
            } => "M5".into(),
            Block::Background {
                background_type: BackgroundType::Reserved3,
            } => "M6".into(),
            Block::Block => "B0".into(),
            Block::Dark => "A0".into(),
            Block::Empty => "00".into(),
            Block::Exit => "E0".into(),
            Block::Key => "IK".into(),
            Block::Lock => "BK".into(),
            Block::Note { text } => format!("Note: {}", text).into(),
            Block::OneWayWall {
                direction: Direction::Down,
            } => "OD".into(),
            Block::OneWayWall {
                direction: Direction::Up,
            } => "OU".into(),
            Block::OneWayWall {
                direction: Direction::Left,
            } => "OL".into(),
            Block::OneWayWall {
                direction: Direction::Right,
            } => "OR".into(),
            Block::PipeIn => "CI".into(),
            Block::PipeOut => "CO".into(),
            Block::PipePhase => "CP".into(),
            Block::PipeSolid => "CS".into(),
            Block::Player => "X0".into(),
            Block::PowerUpBurrow => "P0".into(),
            Block::PowerUpRecall => "P1".into(),
            Block::SecretExit => "E1".into(),
            Block::Scaffold => "D0".into(),
            Block::Switch => "S0".into(),
            Block::SwitchCeiling => "S1".into(),
            Block::ToggleBlock { solid: true } => "T0".into(),
            Block::ToggleBlock { solid: false } => "T1".into(),
            Block::Torch => "D1".into(),
            Block::Wire => "WR".into(),
        }
	}
}
*/


pub struct LevelBuilder {
    level_data: Vec<Block>,
    is_dark: bool,
    background: BackgroundType,
}

impl LevelBuilder {
    pub fn new() -> Self {
        LevelBuilder {
            level_data: vec![Block::Empty; LEVEL_WIDTH * LEVEL_HEIGHT],
            is_dark: false,
            background: BackgroundType::Cobble,
        }
    }

    pub fn set_dark(&mut self, val: bool) {
        self.is_dark = val;
    }
	
	pub fn get_dark(&self) -> bool {
		self.is_dark
	}

    pub fn add_block(&mut self, i: usize, block: Block) {
        *self.level_data.get_mut(i).unwrap() = block;
    }

    pub fn get_level_data(&self) -> &[Block] {
        &self.level_data
    }

    pub fn render_image(&self) -> image::DynamicImage {
        let mut img = match self.background {
            BackgroundType::Cobble => image::load_from_memory(M0_BG).unwrap(), //TODO: LAZY_STATIC
            _ => unimplemented!(),
        }
        .resize(1920, 1080, image::FilterType::Nearest); //TODO: Choose best filter; ,image::FilterType::CatmullRom
                                                         //TODO: Image Cache

        img
    }

    pub fn export_level(&self) -> Option<Vec<Block>> {
        let mut to_insert = Vec::with_capacity(2);
        if self.is_dark {
            to_insert.push(Block::Dark);
        }

        if self.background != BackgroundType::Cobble {
            to_insert.push(Block::Background {
                background_type: self.background.clone(),
            });
        }

        let data = self
            .level_data
            .iter()
            .map(|block| {
                if !to_insert.is_empty() && block == &Block::Empty {
                    to_insert.pop().unwrap_or_else(|| unreachable!())
                } else {
                    block.clone()
                }
            })
            .collect();

        if to_insert.is_empty() {
            Some(data)
        } else {
            None
        }
    }

	/// Imports a block array
    pub fn import(&mut self, level: &[Block]) {
		self.is_dark = false;
		self.background = BackgroundType::Cobble;
		
        for (i, block) in level.iter().enumerate() {
            let block = match block {
                Block::Background { background_type } => {
                    self.background = background_type.clone();
                    Block::Empty
                }
                Block::Dark => {
                    self.set_dark(true);
                    Block::Empty
                }
                b => b.clone(),
            };
            self.level_data[i] = block; //TODO: Zip iter mut
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let img = LevelBuilder::new().render_image();
        img.save("test.png");
    }
}
