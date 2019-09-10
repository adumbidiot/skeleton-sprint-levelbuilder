mod as3;
mod block;

pub use as3::decode_as3;
pub use as3::encode_as3;
pub use block::BackgroundType;
pub use block::Block;
pub use block::Direction;
use std::borrow::Cow;

pub const LEVEL_WIDTH: usize = 32;
pub const LEVEL_HEIGHT: usize = 18;

pub const M0_BG: &[u8] = include_bytes!("../assets/M0.png");

pub fn decode_lbl(data: &str) -> Option<Vec<Block>> {
    data.lines().map(|s| Block::from_lbl(s)).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileFormat {
    LBL,
    AS3,
}

pub fn guess_format(data: &str) -> Option<FileFormat> {
    let mut iter = data.trim().lines();
    let first = iter.next()?;

    if Block::from_lbl(first).is_some() {
        return Some(FileFormat::LBL);
    }

    if first.starts_with("lvlArray") {
        return Some(FileFormat::AS3);
    }

    None
}

pub fn decode_any(data: &str) -> Option<Vec<Block>> {
	let fmt = guess_format(data)?;
	match fmt {
		FileFormat::LBL => decode_lbl(data),
		FileFormat::AS3 => decode_as3(data).ok(),
	}
}

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

    #[test]
    fn as3_decode() {
        let file_data = std::fs::read_to_string("kitchen_sink_as3.txt").unwrap();
        let _data = decode_as3(&file_data).unwrap();
    }
}
