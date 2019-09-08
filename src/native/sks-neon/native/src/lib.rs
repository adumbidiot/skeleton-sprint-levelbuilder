#[macro_use]
extern crate neon;
extern crate sks;

mod util;

use neon::prelude::*;
use sks::*;

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

fn block_array_to_js_array<'a>(mut cx: FunctionContext<'a>, blocks: &[Block]) -> JsResult<'a, JsValue> {
	let js_array = JsArray::new(&mut cx, blocks.len() as u32);
    for (i, block) in blocks.iter().enumerate() {
        let data_str = util::block_to_builder_internal(&block);
        let string = cx.string(data_str);
        js_array.set(&mut cx, i as u32, string)?;
    }
	Ok(js_array.upcast())
}

fn encode_as3(mut cx: FunctionContext) -> JsResult<JsValue> {
    let level_str = cx.argument::<JsString>(0)?.value();
    let array: Option<Vec<Block>> = cx
        .argument::<JsArray>(1)?
        .to_vec(&mut cx)?
        .iter()
        .map(|el| {
            el.downcast::<JsString>()
                .ok()
                .and_then(|el| Block::from_lbl(&el.value()))
        })
        .collect();

    let array = match array {
        Some(a) => a,
        None => return Ok(cx.null().upcast()),
    };

    let output = sks::encode_as3(&level_str, &array);
    Ok(cx.string(&output).upcast())
}

fn decode_any(mut cx: FunctionContext) -> JsResult<JsValue>{
	let raw = cx.argument::<JsString>(0)?.value();
	let input = raw.trim();
	
	let format = match sks::guess_format(input){
		Some(f) => f,
		None => return Ok(cx.null().upcast()),
	};
	
	let level = match format {
		sks::FileFormat::LBL => {
			sks::decode_lbl(input)
		},
		sks::FileFormat::AS3 => {
			sks::decode_as3(input).ok()
		}
	};
	
	let level = match level {
		Some(blocks) => blocks,
		None => return Ok(cx.null().upcast()),
	};
	
	let js_array = block_array_to_js_array(cx, &level)?;
	Ok(js_array.upcast())
}

declare_types! {
    pub class JsLevelBuilder for LevelBuilder {
        init(mut cx) {
            Ok(sks::LevelBuilder::new())
        }

        method getImage(mut cx) {
            use std::convert::TryInto;

            let this = cx.this();
            let img_data = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                lvlbuilder.render_image().to_rgba().into_raw()
            };

            let js_img = {
                let mut js_img = JsArrayBuffer::new(&mut cx, img_data.len().try_into().unwrap())?;
                let guard = cx.lock();
                for (i, byte) in js_img.borrow_mut(&guard).as_mut_slice().iter_mut().enumerate() {
                    *byte = img_data[i];
                }

                js_img
            };

            Ok(js_img.upcast())
        }

        /*
        method add_block(mut cx) {
            let this = cx.this();
            let msg = {
                let guard = cx.lock();
                let block = this.borrow(&guard);
            };

            Ok(cx.null().upcast())
        }
        */
    }
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("export1DPatch", export_1d_patch)?;

    cx.export_function("encodeBlockLBL", encode_block_lbl)?;
    cx.export_function("encodeAS3", encode_as3)?;
	
	cx.export_function("decode", decode_any)?;

    cx.export_class::<JsLevelBuilder>("LevelBuilder")
});
