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
