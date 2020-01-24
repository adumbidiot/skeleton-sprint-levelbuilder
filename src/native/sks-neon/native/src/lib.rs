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

/// Converts from the lvlbuilders old internal rep into lbl format
fn encode_block_lbl(mut cx: FunctionContext) -> JsResult<JsValue> {
    let block_str = cx.argument::<JsString>(0)?.value();
	match util::builder_internal_to_block(&block_str) {
		Some(v) => Ok(cx.string(v.as_lbl()).upcast()),
		 None => {
			println!("[sks_rust::encode_block_lbl] Unknown: {}", &block_str);
                Ok(cx.null().upcast())
			}
	}
}

fn block_array_to_js_array<'a, T: neon::object::This>(
    mut cx: CallContext<'a, T>,
    blocks: &[Block],
) -> JsResult<'a, JsValue> {
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
                .and_then(|el| Block::from_lbl(&el.value()).ok())
        })
        .collect();

    let array = match array {
        Some(a) => a,
        None => return Ok(cx.null().upcast()),
    };

    let output = sks::encode_as3(&level_str, &array);
    Ok(cx.string(&output).upcast())
}

fn decode_any(mut cx: FunctionContext) -> JsResult<JsValue> {
    let input = cx.argument::<JsString>(0)?.value();

    let level = match sks::decode_any(&input) {
        Some(f) => f,
        None => return Ok(cx.null().upcast()),
    };

    let js_array = block_array_to_js_array(cx, &level)?;
    Ok(js_array.upcast())
}

declare_types! {
    pub class JsLevelBuilder for LevelBuilder {
        init(_cx) {
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

        method getLevelData(mut cx) {
            let this = cx.this();
            let level_data = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                let data = lvlbuilder.get_level_data().to_vec();
                drop(lvlbuilder);
                block_array_to_js_array(cx, &data)?
            };

            Ok(level_data)
        }

        method addBlock(mut cx) {
            let i = cx.argument::<JsNumber>(0)?.value() as usize;
            let block = cx.argument::<JsString>(1)?.value();
            let mut this = cx.this();
            let msg = {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.add_block(i, util::builder_internal_to_block(&block).unwrap());
            };

            Ok(cx.null().upcast())
        }

        method exportLevel(mut cx){
            let this = cx.this();
            let level_data = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                let data = lvlbuilder.export_level().unwrap();
                drop(lvlbuilder);
                block_array_to_js_array(cx, &data)?
            };

            Ok(level_data)
        }

        method setDark(mut cx){
            let val = cx.argument::<JsBoolean>(0)?.value();
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.set_dark(val);
            }
            Ok(cx.undefined().upcast())
        }

        method getDark(mut cx){
            let this = cx.this();
            let dark = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                lvlbuilder.get_dark()
            };
            Ok(cx.boolean(dark).upcast())
        }

        method import(mut cx){
            let val = cx.argument::<JsString>(0)?.value();
            let data = match sks::decode_any(&val) {
                Some(d) => d,
                None => return Ok(cx.boolean(false).upcast()),
            };
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);

                lvlbuilder.import(&data);
            }
            Ok(cx.boolean(true).upcast())
        }
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
