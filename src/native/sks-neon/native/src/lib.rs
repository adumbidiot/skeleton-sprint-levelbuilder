mod util;

use neon::prelude::*;
pub use skeleton_sprint_levelbuilder::sks;
use sks::format::LevelNumber;

pub type LevelBuilder = skeleton_sprint_levelbuilder::App;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello sks-neon"))
}

fn get_frame<'a>(
    cx: &mut CallContext<'a, JsLevelBuilder>,
) -> Result<Handle<'a, JsValue>, neon::result::Throw> {
    let mut this = cx.this();

    let img_data = {
        let guard = cx.lock();
        let mut lvlbuilder = this.borrow_mut(&guard);
        lvlbuilder.get_raw_image()
    };

    let img_data = match img_data {
        Ok(data) => data,
        Err(e) => return cx.throw_error(&e.to_string()),
    };

    let img_data = util::rgba_image_to_image_data(cx, img_data)?;
    let canvas = util::img_data_to_canvas(cx, img_data)?;

    Ok(canvas.upcast())
}

fn get_image<'a>(
    cx: &mut CallContext<'a, JsLevelBuilder>,
) -> Result<Handle<'a, JsValue>, neon::result::Throw> {
    let mut this = cx.this();

    let img_data = {
        let guard = cx.lock();
        let mut lvlbuilder = this.borrow_mut(&guard);
        lvlbuilder.get_level_image()
    };

    let img_data = match img_data {
        Ok(data) => data.into_rgba8(),
        Err(_e) => return cx.throw_error("SKS render error"),
    };

    let img_data = util::rgba_image_to_image_data(cx, img_data)?;
    let canvas = util::img_data_to_canvas(cx, img_data)?;

    Ok(canvas.upcast())
}

declare_types! {
    pub class JsLevelBuilder for LevelBuilder {
        init(_cx) {
            Ok(LevelBuilder::new().expect("sks Levelbuilder"))
        }

        method getFrame(mut cx) {
            get_frame(&mut cx)
        }

        method getImage(mut cx) {
            get_image(&mut cx)
        }

        method emitRecievedChar(mut cx) {
            let key = cx.argument::<JsString>(0)?.value().chars().next().unwrap();

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);

                lvlbuilder.emit_recieved_char(key);
            }

            Ok(cx.undefined().upcast())
        }

        method emitKeyboardEvent(mut cx) {
            let kind = cx.argument::<JsString>(0)?.value();
            let key = cx.argument::<JsNumber>(1)?.value();

            // TODO: Modifiers?

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);

                match kind.as_str() {
                    "down" => {
                        lvlbuilder.emit_keyboard_key_down(key as u64);
                    },
                    "up" => {
                        lvlbuilder.emit_keyboard_key_up(key as u64);
                    },
                    unknown => panic!("Unknown: {:#?}", unknown),
                }
            }

            Ok(cx.undefined().upcast())
        }

        method emitMouseButtonEvent(mut cx) {
            let button = cx.argument::<JsString>(0)?.value();
            let kind = cx.argument::<JsString>(1)?.value();

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);

                match (button.as_str(), kind.as_str()) {
                    ("left", "down") => {
                        lvlbuilder.emit_left_mouse_button_down();
                    },
                    ("right", "down") => {
                        lvlbuilder.emit_right_mouse_button_down();
                    },
                    ("left", "up") => {
                        lvlbuilder.emit_left_mouse_button_up();
                    },
                    ("right", "up") => {
                        lvlbuilder.emit_right_mouse_button_up();
                    },
                    unknown => panic!("Unknown: {:#?}", unknown),
                }
            }

            Ok(cx.undefined().upcast())
        }

        method update(mut cx) {
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.update();
            }

            Ok(cx.undefined().upcast())
        }

        method updateMousePosition(mut cx) {
            let x = cx.argument::<JsNumber>(0)?.value();
            let y = cx.argument::<JsNumber>(1)?.value();

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.update_mouse_position(x, y);
            }

            Ok(cx.undefined().upcast())
        }

        method export(mut cx) {
            let block = cx.argument_opt(0).map(|v| Ok(v.to_string(&mut cx)?.value())).transpose()?;
            let block = block.unwrap_or_else(|| "lbl".into());

            let format = match block.as_str() {
                "as3" => {
                    sks::format::FileFormat::As3
                },
                "lbl" => {
                    sks::format::FileFormat::Lbl
                },
                o => {
                    return cx.throw_error(&format!("Unknown Option: {}", o));
                }
            };

             let mut this = cx.this();

             let ret = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow_mut(&guard);
                sks::format::encode(&lvlbuilder.export().unwrap(), &format, Some(&lvlbuilder.get_level_number().unwrap_or(LevelNumber::Identifier("x".into())))).unwrap()
            };
            Ok(cx.string(&ret).upcast())
        }

        method setLevel(mut cx){
            let arg = cx.argument::<JsValue>(0)?;
            let level_number = if let Ok(v) = arg.downcast::<JsNumber>() {
                LevelNumber::Number(v.value() as usize)
            }else if let Ok(v) = arg.downcast::<JsString>() {
                LevelNumber::String(v.value())
            }else{
                return cx.throw_error("Invalid Arg");
            };

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);

                lvlbuilder.set_level_number(level_number);
            }

            Ok(cx.undefined().upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;

    cx.export_class::<JsLevelBuilder>("LevelBuilder")
});
