#[macro_use]
extern crate neon;

use neon::prelude::*;

fn export_1d_patch(mut cx: FunctionContext) -> JsResult<JsString> {
    let v = cx.argument::<JsArray>(0)?;
    let v_str = v.to_string(&mut cx)?;
    Ok(v_str)
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

///Converts from the lvlbuilders old internal rep into lbl format
fn encode_block_lbl(mut cx: FunctionContext) -> JsResult<JsValue> {
    let block_str = cx.argument::<JsString>(0)?.value();
    match block_str.as_str() {
        "block" => Ok(cx.string("B0").upcast()),
        "block_key" => Ok(cx.string("BK").upcast()),
        "cobble_bg" => Ok(cx.string("M0").upcast()),
        "concrete_bg" => Ok(cx.string("M3").upcast()),
        "decoration_scaffold" => Ok(cx.string("D0").upcast()),
        "decoration_sconce" => Ok(cx.string("D1").upcast()),
        "exit" => Ok(cx.string("E0").upcast()),
        "item_key" => Ok(cx.string("IK").upcast()),
        "main" => Ok(cx.string("X0").upcast()),
        "mask_circle" => Ok(cx.string("A0").upcast()),
        "null" => Ok(cx.string("00").upcast()),
        "onewaywalldown" => Ok(cx.string("OD").upcast()),
        "onewaywallleft" => Ok(cx.string("OL").upcast()),
        "onewaywallright" => Ok(cx.string("OR").upcast()),
        "onewaywallup" => Ok(cx.string("OU").upcast()),
        "pipe_in" => Ok(cx.string("CI").upcast()),
        "pipe_out" => Ok(cx.string("CO").upcast()),
        "pipe_phase" => Ok(cx.string("CP").upcast()),
        "pipe_solid" => Ok(cx.string("CS").upcast()),
        "powerupburrow" => Ok(cx.string("P0").upcast()),
        "poweruprecall" => Ok(cx.string("P1").upcast()),
        "skullfall_bg" => Ok(cx.string("M2").upcast()),
        "secretexit" => Ok(cx.string("E1").upcast()),
        "switch" => Ok(cx.string("S0").upcast()),
        "switchceiling" => Ok(cx.string("S1").upcast()),
        "toggleblocksolid" => Ok(cx.string("T0").upcast()),
        "toggleblockphase" => Ok(cx.string("T1").upcast()),
        "undefined1" => Ok(cx.string("M4").upcast()),
        "undefined2" => Ok(cx.string("M5").upcast()),
        "undefined3" => Ok(cx.string("M6").upcast()),
        "waterfall_bg" => Ok(cx.string("M1").upcast()),
        "wirered" => Ok(cx.string("WR").upcast()),
        block_str => {
            if block_str.starts_with("Note:") {
                Ok(cx.string(block_str).upcast())
            } else {
                println!("[sks_rust::encode_block_lbl] Unknown: {}", &block_str);
                Ok(cx.null().upcast())
            }
        }
    }
}

/// Converts from lbl into the old levelbuilder's internal rep
fn decode_block_lbl(mut cx: FunctionContext) -> JsResult<JsValue> {
    let data_str = cx.argument::<JsString>(0)?.value();
    match Block::from_lbl(&data_str) {
        Some(b) => match b {
            Block::Block => Ok(cx.string("block").upcast()),
            Block::Background {
                background_type: BackgroundType::Cobble,
            } => Ok(cx.string("cobble_bg").upcast()),
            Block::Background {
                background_type: BackgroundType::Waterfall,
            } => Ok(cx.string("waterfall_bg").upcast()),
            Block::Background {
                background_type: BackgroundType::Skullfall,
            } => Ok(cx.string("skullfall_bg").upcast()),
            Block::Background {
                background_type: BackgroundType::Concrete,
            } => Ok(cx.string("concrete_bg").upcast()),
            Block::Background {
                background_type: BackgroundType::Reserved1,
            } => Ok(cx.string("undefined1").upcast()),
            Block::Background {
                background_type: BackgroundType::Reserved2,
            } => Ok(cx.string("undefined2").upcast()),
            Block::Background {
                background_type: BackgroundType::Reserved3,
            } => Ok(cx.string("undefined3").upcast()),
			Block::Dark => Ok(cx.string("mask_circle").upcast()),
            Block::Empty => Ok(cx.string("null").upcast()),
            Block::Exit => Ok(cx.string("exit").upcast()),
            Block::Key => Ok(cx.string("item_key").upcast()),
            Block::Lock => Ok(cx.string("block_key").upcast()),
            Block::Note { text } => Ok(cx.string(&format!("Note:{}", text)).upcast()),
            Block::OneWayWall {
                direction: Direction::Up,
            } => Ok(cx.string("onewaywallup").upcast()),
            Block::OneWayWall {
                direction: Direction::Down,
            } => Ok(cx.string("onewaywalldown").upcast()),
            Block::OneWayWall {
                direction: Direction::Left,
            } => Ok(cx.string("onewaywallleft").upcast()),
            Block::OneWayWall {
                direction: Direction::Right,
            } => Ok(cx.string("onewaywallright").upcast()),
            Block::PipeIn => Ok(cx.string("pipe_in").upcast()),
            Block::PipeOut => Ok(cx.string("pipe_out").upcast()),
            Block::PipePhase => Ok(cx.string("pipe_phase").upcast()),
            Block::PipeSolid => Ok(cx.string("pipe_solid").upcast()),
            Block::Player => Ok(cx.string("main").upcast()),
            Block::PowerUpBurrow => Ok(cx.string("powerupburrow").upcast()),
            Block::PowerUpRecall => Ok(cx.string("poweruprecall").upcast()),
			Block::SecretExit => Ok(cx.string("secretexit").upcast()),
            Block::Scaffold => Ok(cx.string("decoration_scaffold").upcast()),
            Block::Switch => Ok(cx.string("switch").upcast()),
			Block::SwitchCeiling => Ok(cx.string("switchceiling").upcast()),
            Block::ToggleBlock { solid: true } => Ok(cx.string("toggleblocksolid").upcast()),
            Block::ToggleBlock { solid: false } => Ok(cx.string("toggleblockphase").upcast()),
            Block::Torch => Ok(cx.string("decoration_sconce").upcast()),
            Block::Wire => Ok(cx.string("wirered").upcast()),
        },
        None => {
            println!("[sks_rust::decode_block_lbl] Unknown: {}", &data_str);
            Ok(cx.null().upcast())
        }
    }
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
enum BackgroundType {
    Cobble,
    Waterfall,
    Skullfall,
    Concrete,
    Reserved1,
    Reserved2,
    Reserved3,
}

#[derive(Debug)]
enum Block {
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
    /*
    pub fn as_lbl(&self) -> &str {
        match self {
            Block::Block => "B0",
            Block::Empty => "00",
            Block::Exit => "E0",
            Block::Switch => "S0",
            Block::Player => "X0",
            Block::ToggleBlock {solid: true} => "T0",
            Block::ToggleBlock {solid: false} => "T1",
            Block::Torch => "D1",
        }
    }
    */
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("export1DPatch", export_1d_patch)?;
    cx.export_function("encodeBlockLBL", encode_block_lbl)?;
    cx.export_function("decodeBlockLBL", decode_block_lbl)
});
