use neon::prelude::*;
use std::convert::TryInto;
use skeleton_sprint_levelbuilder::RgbaImage;

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
    img: RgbaImage,
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
