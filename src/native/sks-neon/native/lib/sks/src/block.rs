/// The directions something could face
#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// The types fo backgrounds
#[derive(Debug)]
pub enum BackgroundType {
    Cobble,
    Waterfall,
    Skullfall,
    Concrete,
    Reserved1,
    Reserved2,
    Reserved3,
}

/// An entity that occupies a space in lbl representation.
/// Also the internal rep  of a "block" in this library.
#[derive(Debug)]
pub enum Block {
    Background { background_type: BackgroundType },
    Block,
    Dark,
    Empty,
    Exit,
    Key,
    Lock,
    Note { text: String },
    Scaffold,
    SecretExit,
    Switch,
    SwitchCeiling,
    OneWayWall { direction: Direction },
    PipeIn,
    PipeOut,
    PipePhase,
    PipeSolid,
    Player,
    PowerUpBurrow,
    PowerUpRecall,
    ToggleBlock { solid: bool },
    Torch,
    Wire,
}

impl Block {
    /// Decodes an lbl string to a block, if valid
    pub fn from_lbl(data: &str) -> Option<Block> {
        match data {
            "00" => Some(Block::Empty),
            "A0" => Some(Block::Dark),
            "B0" => Some(Block::Block),
            "BK" => Some(Block::Lock),
            "CI" => Some(Block::PipeIn),
            "CO" => Some(Block::PipeOut),
            "CP" => Some(Block::PipePhase),
            "CS" => Some(Block::PipeSolid),
            "D0" => Some(Block::Scaffold),
            "D1" => Some(Block::Torch),
            "E0" => Some(Block::Exit),
            "E1" => Some(Block::SecretExit),
            "IK" => Some(Block::Key),
            "M0" => Some(Block::Background {
                background_type: BackgroundType::Cobble,
            }),
            "M1" => Some(Block::Background {
                background_type: BackgroundType::Waterfall,
            }),
            "M2" => Some(Block::Background {
                background_type: BackgroundType::Skullfall,
            }),
            "M3" => Some(Block::Background {
                background_type: BackgroundType::Concrete,
            }),
            "M4" => Some(Block::Background {
                background_type: BackgroundType::Reserved1,
            }),
            "M5" => Some(Block::Background {
                background_type: BackgroundType::Reserved2,
            }),
            "M6" => Some(Block::Background {
                background_type: BackgroundType::Reserved3,
            }),
            "OD" => Some(Block::OneWayWall {
                direction: Direction::Down,
            }),
            "OL" => Some(Block::OneWayWall {
                direction: Direction::Left,
            }),
            "OR" => Some(Block::OneWayWall {
                direction: Direction::Right,
            }),
            "OU" => Some(Block::OneWayWall {
                direction: Direction::Up,
            }),
            "P0" => Some(Block::PowerUpBurrow),
            "P1" => Some(Block::PowerUpRecall),
            "S0" => Some(Block::Switch),
            "S1" => Some(Block::SwitchCeiling),
            "T0" => Some(Block::ToggleBlock { solid: true }),
            "T1" => Some(Block::ToggleBlock { solid: false }),
            "X0" => Some(Block::Player),
            "WR" => Some(Block::Wire),
            data => {
                let note_prefix = "Note:";
                if data.starts_with(note_prefix) {
                    Some(Block::Note {
                        text: String::from(&data[note_prefix.len()..]),
                    })
                } else {
                    None
                }
            }
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block::Empty
    }
}
