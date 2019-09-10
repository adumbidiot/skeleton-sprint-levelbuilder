use sks::{BackgroundType, Block, Direction};
use std::borrow::Cow;

/// Converts a block into the old lvlbuilder rep
pub fn block_to_builder_internal(b: &Block) -> Cow<'static, str> {
    match b {
        Block::Block => "block".into(),
        Block::Background {
            background_type: BackgroundType::Cobble,
        } => "cobble_bg".into(),
        Block::Background {
            background_type: BackgroundType::Waterfall,
        } => "waterfall_bg".into(),
        Block::Background {
            background_type: BackgroundType::Skullfall,
        } => "skullfall_bg".into(),
        Block::Background {
            background_type: BackgroundType::Concrete,
        } => "concrete_bg".into(),
        Block::Background {
            background_type: BackgroundType::Reserved1,
        } => "undefined1".into(),
        Block::Background {
            background_type: BackgroundType::Reserved2,
        } => "undefined2".into(),
        Block::Background {
            background_type: BackgroundType::Reserved3,
        } => "undefined3".into(),
        Block::Dark => "mask_circle".into(),
        Block::Empty => "null".into(),
        Block::Exit => "exit".into(),
        Block::Key => "item_key".into(),
        Block::Lock => "block_key".into(),
        Block::Note { text } => format!("Note:{}", text).into(),
        Block::OneWayWall {
            direction: Direction::Up,
        } => "onewaywallup".into(),
        Block::OneWayWall {
            direction: Direction::Down,
        } => "onewaywalldown".into(),
        Block::OneWayWall {
            direction: Direction::Left,
        } => "onewaywallleft".into(),
        Block::OneWayWall {
            direction: Direction::Right,
        } => "onewaywallright".into(),
        Block::PipeIn => "pipe_in".into(),
        Block::PipeOut => "pipe_out".into(),
        Block::PipePhase => "pipe_phase".into(),
        Block::PipeSolid => "pipe_solid".into(),
        Block::Player => "main".into(),
        Block::PowerUpBurrow => "powerupburrow".into(),
        Block::PowerUpRecall => "poweruprecall".into(),
        Block::SecretExit => "secretexit".into(),
        Block::Scaffold => "decoration_scaffold".into(),
        Block::Switch => "switch".into(),
        Block::SwitchCeiling => "switchceiling".into(),
        Block::ToggleBlock { solid: true } => "toggleblocksolid".into(),
        Block::ToggleBlock { solid: false } => "toggleblockphase".into(),
        Block::Torch => "decoration_sconce".into(),
        Block::Wire => "wirered".into(),
    }
}

/// Converts from levelbuilder format to a block
pub fn builder_internal_to_block(block_str: &str) -> Option<Block> {
    match block_str {
        "block" => Some(Block::Block),
        "block_key" => Some(Block::Lock),
        "cobble_bg" => Some(Block::Background {
            background_type: BackgroundType::Cobble,
        }),
        "concrete_bg" => Some(Block::Background {
            background_type: BackgroundType::Concrete,
        }),
        "decoration_scaffold" => Some(Block::Scaffold),
        "decoration_sconce" => Some(Block::Torch),
        "exit" => Some(Block::Exit),
        "item_key" => Some(Block::Key),
        "main" => Some(Block::Player),
        "mask_circle" => Some(Block::Dark),
        "null" => Some(Block::Empty),
        "onewaywalldown" => Some(Block::OneWayWall {
            direction: Direction::Down,
        }),
        "onewaywallleft" => Some(Block::OneWayWall {
            direction: Direction::Left,
        }),
        "onewaywallright" => Some(Block::OneWayWall {
            direction: Direction::Right,
        }),
        "onewaywallup" => Some(Block::OneWayWall {
            direction: Direction::Up,
        }),
        "pipe_in" => Some(Block::PipeIn),
        "pipe_out" => Some(Block::PipeOut),
        "pipe_phase" => Some(Block::PipePhase),
        "pipe_solid" => Some(Block::PipeSolid),
        "powerupburrow" => Some(Block::PowerUpBurrow),
        "poweruprecall" => Some(Block::PowerUpRecall),
        "skullfall_bg" => Some(Block::Background {
            background_type: BackgroundType::Skullfall,
        }),
        "secretexit" => Some(Block::SecretExit),
        "switch" => Some(Block::Switch),
        "switchceiling" => Some(Block::SwitchCeiling),
        "toggleblocksolid" => Some(Block::ToggleBlock { solid: true }),
        "toggleblockphase" => Some(Block::ToggleBlock { solid: false }),
        "undefined1" => Some(Block::Background {
            background_type: BackgroundType::Reserved1,
        }),
        "undefined2" => Some(Block::Background {
            background_type: BackgroundType::Reserved2,
        }),
        "undefined3" => Some(Block::Background {
            background_type: BackgroundType::Reserved3,
        }),
        "waterfall_bg" => Some(Block::Background {
            background_type: BackgroundType::Waterfall,
        }),
        "wirered" => Some(Block::Wire),
        block_str => {
            let note_frag = "Note:";
            if block_str.starts_with(note_frag) {
                Some(Block::Note {
                    text: block_str[note_frag.len()..].into(),
                })
            } else {
                None
            }
        }
    }
}
