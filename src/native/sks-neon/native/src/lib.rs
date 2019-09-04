#[macro_use]
extern crate neon;
extern crate sks;

use neon::prelude::*;
use sks::*;
use std::borrow::Cow;

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

fn block_to_builder_internal(b: &Block) -> Cow<'static, str> {
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

/// Converts from lbl into the old levelbuilder's internal rep
fn decode_block_lbl(mut cx: FunctionContext) -> JsResult<JsValue> {
    let data_str = cx.argument::<JsString>(0)?.value();
    match Block::from_lbl(&data_str) {
        Some(b) => Ok(cx.string(block_to_builder_internal(&b)).upcast()),
        None => {
            println!("[sks_rust::decode_block_lbl] Unknown: {}", &data_str);
            Ok(cx.null().upcast())
        }
    }
}

fn decode_as3(mut cx: FunctionContext) -> JsResult<JsValue> {
    let data_str = cx.argument::<JsString>(0)?.value();
    let level = match sks::decode_as3(&data_str) {
        Ok(data) => data,
        Err(_) => return Ok(cx.null().upcast()),
    };

    let js_array = JsArray::new(&mut cx, level.len() as u32);
    for (i, block) in level.iter().enumerate() {
        let data_str = block_to_builder_internal(&block);
        let string = cx.string(data_str);
        js_array.set(&mut cx, i as u32, string)?;
    }

    Ok(js_array.upcast())
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("export1DPatch", export_1d_patch)?;
    cx.export_function("encodeBlockLBL", encode_block_lbl)?;
    cx.export_function("decodeBlockLBL", decode_block_lbl)?;
    cx.export_function("decodeAS3", decode_as3)
});
