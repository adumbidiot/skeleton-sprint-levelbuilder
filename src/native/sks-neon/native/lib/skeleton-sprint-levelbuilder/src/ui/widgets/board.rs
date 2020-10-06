use iced_core::Rectangle;
use iced_graphics::{Backend, Defaults, Primitive, Renderer};
use iced_native::{
    layout, mouse, Background, Color, Element, Hasher, Layout, Length, Point, Size, Widget,
};
use std::convert::TryFrom;
use std::hash::Hash;

pub struct Board<'a> {
    level: &'a crate::Level,
    background_image: &'a iced_native::image::Handle,
    iced_block_map: &'a crate::IcedBlockMap,

    grid: bool,
}

impl<'a> Board<'a> {
    pub fn new(
        level: &'a crate::Level,
        background_image: &'a iced_native::image::Handle,
        iced_block_map: &'a crate::IcedBlockMap,
    ) -> Self {
        Board {
            level,
            background_image,
            iced_block_map,
            grid: true,
        }
    }

    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }
}

impl<'a, Message, B> Widget<Message, Renderer<B>> for Board<'a>
where
    B: Backend,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer<B>, _limits: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(
            // f32::from(self.radius) * 2.0,
            // f32::from(self.radius) * 2.0,
            1920.0, 1080.0,
        ))
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.level.hash(state);
        self.grid.hash(state);
        // self.iced_block_map.hash(state);
    }

    fn draw(
        &self,
        _renderer: &mut Renderer<B>,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> (Primitive, mouse::Interaction) {
        let layout_bounds = layout.bounds();

        // one per block + one for bg + one for grid overlay
        let mut primitives = Vec::with_capacity(sks::LEVEL_SIZE + 2);
        let block_size = layout_bounds.width as usize / sks::LEVEL_WIDTH;

        primitives.push(Primitive::Image {
            handle: self.background_image.clone(),
            bounds: layout.bounds(),
        });

        for (i, block) in self.level.get_level_data().iter().enumerate() {
            if !block.is_empty() {
                let handle = self.iced_block_map.get(block.clone());
                let x = ((i % sks::LEVEL_WIDTH) * block_size) as f32 + layout_bounds.x;
                let y = ((i / sks::LEVEL_WIDTH) * block_size) as f32 + layout_bounds.y;
                let bounds = Rectangle {
                    x,
                    y,
                    width: block_size as f32,
                    height: block_size as f32,
                };
                primitives.push(Primitive::Image { handle, bounds });
            }
        }

        if self.grid {
            let mut grid_primitives = Vec::with_capacity(sks::LEVEL_SIZE);
            for i in 0..sks::LEVEL_SIZE {
                let x = ((i % sks::LEVEL_WIDTH) * block_size) as f32 + layout_bounds.x;
                let y = ((i / sks::LEVEL_WIDTH) * block_size) as f32 + layout_bounds.y;
                let bounds = Rectangle {
                    x,
                    y,
                    width: block_size as f32,
                    height: block_size as f32,
                };

                grid_primitives.push(Primitive::Quad {
                    bounds,
                    background: Background::Color(Color::TRANSPARENT),
                    border_radius: 0,
                    border_width: 2,
                    border_color: Color::BLACK,
                });
            }

            primitives.push(Primitive::Clip {
                bounds: layout_bounds,
                offset: Default::default(),
                content: Box::new(Primitive::Group {
                    primitives: grid_primitives,
                }),
            });
        }

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }
}

impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for Board<'a>
where
    B: Backend,
{
    fn into(self) -> Element<'a, Message, Renderer<B>> {
        Element::new(self)
    }
}
