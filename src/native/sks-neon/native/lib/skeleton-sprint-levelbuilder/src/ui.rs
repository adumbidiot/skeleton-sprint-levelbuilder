use crate::ConrodBlockMap;
use conrod_core::widget;
use conrod_core::widget_ids;
use conrod_core::Positionable;
use conrod_core::Sizeable;
use conrod_core::Widget;
use sks::block::Block as SksBlock;
use std::convert::TryFrom;

widget_ids! {
    pub struct Ids {
        background,

        blocks[],

        text
    }
}

pub fn gui(
    ui: &mut conrod_core::UiCell,
    ids: &mut Ids,
    _font: &conrod_core::text::font::Id,
    level_data: &[SksBlock],
    ui_data: UiData<'_>,
) {
    widget::Image::new(ui_data.background_image)
        .w_h(crate::WINDOW_WIDTH.into(), crate::WINDOW_HEIGHT.into())
        .middle()
        .set(ids.background, ui);

    let block_size = crate::WINDOW_WIDTH / u32::try_from(sks::LEVEL_WIDTH).unwrap();

    let mut id_list_walk = ids.blocks.walk();
    for (i, block) in level_data.iter().enumerate() {
        let ui_block_id = id_list_walk.next(&mut ids.blocks, &mut ui.widget_id_generator());

        let img = ui_data.conrod_block_map.get(block.clone());

        let i_u32 = u32::try_from(i).unwrap();
        let level_width_u32 = u32::try_from(sks::LEVEL_WIDTH).unwrap();
        let x = f64::from((i_u32 / level_width_u32) * block_size);
        let y = f64::from((i_u32 % level_width_u32) * block_size);

        if !block.is_empty() {
            widget::Image::new(img)
                .parent(ids.background)
                .w_h(block_size.into(), block_size.into())
                .parent(ids.background)
                .top_left_with_margins(x, y)
                .set(ui_block_id, ui);
        }
    }

    // widget::Text::new("TEST HELLO 123").set(ids.text, ui);
}

pub struct UiData<'a> {
    pub invalid_block_image: conrod_core::image::Id,
    pub background_image: conrod_core::image::Id,
    pub conrod_block_map: &'a ConrodBlockMap,
}
