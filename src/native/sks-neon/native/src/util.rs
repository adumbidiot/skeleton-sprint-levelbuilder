use neon::prelude::*;
use sks::{BackgroundType, Block, Direction};
use std::borrow::Cow;
use std::convert::TryInto;

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
            let note_frag = "Note: ";
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

pub fn byte_slice_to_js_array_buffer<'a, T: neon::object::This>(
    cx: &mut CallContext<'a, T>,
    bytes: &[u8],
) -> Result<Handle<'a, JsArrayBuffer>, neon::result::Throw> {
    let len = bytes.len().try_into().unwrap();
    let mut js_array_buffer = cx.array_buffer(len)?;

    let guard = cx.lock();

    js_array_buffer
        .borrow_mut(&guard)
        .as_mut_slice::<u8>()
        .iter_mut()
        .zip(bytes.iter())
        .for_each(|(js_byte, rust_byte)| *js_byte = *rust_byte);

    Ok(js_array_buffer)
}

pub fn rgba_image_to_image_data<'a, T: neon::object::This>(
    cx: &mut CallContext<'a, T>,
    img: sks::RgbaImage,
) -> Result<Handle<'a, JsObject>, neon::result::Throw> {
    let dimensions = img.dimensions();
    let img_bytes = img.into_vec();

    let js_array_buffer = byte_slice_to_js_array_buffer(cx, &img_bytes)?;

    let js_img = {
        let global = cx.global();

        let uint8_clamped_array_constructor = global
            .get(cx, "Uint8ClampedArray")?
            .downcast_or_throw::<JsFunction, _>(cx)?;

        let js_array_buffer =
            uint8_clamped_array_constructor.construct(cx, Some(js_array_buffer))?;

        let image_data_constructor = global
            .get(cx, "ImageData")?
            .downcast_or_throw::<JsFunction, _>(cx)?;

        let js_array_buffer = js_array_buffer.upcast::<JsValue>();
        let js_width = cx.number(dimensions.0).upcast::<JsValue>();
        let js_height = cx.number(dimensions.1).upcast::<JsValue>();

        image_data_constructor.construct(cx, vec![js_array_buffer, js_width, js_height])?
    };

    Ok(js_img.downcast_or_throw(cx)?)
}

pub fn img_data_to_canvas<'a, T: neon::object::This>(
    cx: &mut CallContext<'a, T>,
    img_data: Handle<'a, JsObject>,
) -> Result<Handle<'a, JsObject>, neon::result::Throw> {
    let global = cx.global();
    let js_document = global
        .get(cx, "document")?
        .downcast_or_throw::<JsObject, _>(cx)?;

    let create_element_func = js_document
        .get(cx, "createElement")?
        .downcast_or_throw::<JsFunction, _>(cx)?;

    let canvas_str = cx.string("canvas");
    let canvas = create_element_func
        .call(cx, js_document, Some(canvas_str))?
        .downcast_or_throw::<JsObject, _>(cx)?;

    let width_str = cx.string("width");
    let height_str = cx.string("height");

    let width_num = img_data.get(cx, width_str)?;
    let height_num = img_data.get(cx, height_str)?;

    canvas.set(cx, width_str, width_num)?;
    canvas.set(cx, height_str, height_num)?;

    let js_2d = cx.string("2d");
    let context_2d = canvas
        .get(cx, "getContext")?
        .downcast_or_throw::<JsFunction, _>(cx)?
        .call(cx, canvas, Some(js_2d))?
        .downcast_or_throw::<JsObject, _>(cx)?;

    let js_zero = cx.number(0).upcast::<JsValue>();
    context_2d
        .get(cx, "putImageData")?
        .downcast_or_throw::<JsFunction, _>(cx)?
        .call(cx, context_2d, vec![img_data.upcast(), js_zero, js_zero])?;

    Ok(canvas)
}
