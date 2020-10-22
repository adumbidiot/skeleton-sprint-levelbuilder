use crate::ui::get_relative_position;
use iced_core::Rectangle;
use iced_graphics::{Backend, Defaults, Primitive, Renderer};
use iced_native::Clipboard;
use iced_native::Event;
use iced_native::{
    layout, mouse, Background, Color, Element, Hasher, Layout, Length, Point, Size, Widget,
};
use std::convert::TryFrom;
use std::hash::Hash;

pub struct State {
    left_mouse_down: bool,
    right_mouse_down: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            left_mouse_down: false,
            right_mouse_down: false,
        }
    }
}

pub struct Board<'a> {
    level: &'a crate::Level,
    background_image: &'a iced_native::image::Handle,
    iced_block_map: &'a crate::IcedBlockMap,

    active_block: Option<&'a sks::Block>,

    grid: bool,

    state: &'a mut State,
}

impl<'a> Board<'a> {
    pub fn new(
        level: &'a crate::Level,
        background_image: &'a iced_native::image::Handle,
        iced_block_map: &'a crate::IcedBlockMap,
        state: &'a mut State,
    ) -> Self {
        Board {
            level,
            background_image,
            iced_block_map,

            active_block: None,

            grid: true,

            state,
        }
    }

    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    pub fn active_block(mut self, active_block: Option<&'a sks::Block>) -> Self {
        self.active_block = active_block;
        self
    }
}

impl<'a> Board<'a> {}

impl<'a, B> Widget<crate::ui::Message, Renderer<B>> for Board<'a>
where
    B: Backend,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer<B>, limits: &layout::Limits) -> layout::Node {
        // let (width, height) = renderer.dimensions(&self.handle);
        let (width, height) = (crate::WINDOW_WIDTH, crate::WINDOW_HEIGHT);

        let aspect_ratio = crate::WINDOW_WIDTH as f32 / crate::WINDOW_HEIGHT as f32;

        let mut size = limits
            // .width(self.width)
            // .height(self.height)
            .resolve(Size::new(
                crate::WINDOW_WIDTH as f32,
                crate::WINDOW_HEIGHT as f32,
            ));

        let viewport_aspect_ratio = size.width / size.height;

        if viewport_aspect_ratio > aspect_ratio {
            size.width = width as f32 * size.height / height as f32;
        } else {
            size.height = height as f32 * size.width / width as f32;
        }

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<crate::ui::Message>,
        _renderer: &Renderer<B>,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        let layout_bounds = layout.bounds();
        let block_size = layout_bounds.width / sks::LEVEL_WIDTH as f32;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                match button {
                    mouse::Button::Left => {
                        self.state.left_mouse_down = true;
                    }
                    mouse::Button::Right => {
                        self.state.right_mouse_down = true;
                    }
                    _ => {}
                }

                if layout_bounds.contains(cursor_position) {
                    let rel_pos = get_relative_position(&layout_bounds, &cursor_position);

                    let index = (rel_pos.x / block_size) as usize
                        + ((rel_pos.y / block_size) as usize * sks::LEVEL_WIDTH);

                    match button {
                        mouse::Button::Left => {
                            if let Some(block) = self.active_block.cloned() {
                                messages.push(crate::ui::Message::AddBlock { index, block });
                            }
                        }
                        mouse::Button::Right => {
                            messages.push(crate::ui::Message::AddBlock {
                                index,
                                block: sks::Block::Empty,
                            });
                        }
                        _ => {}
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => match button {
                mouse::Button::Left => {
                    self.state.left_mouse_down = false;
                }
                mouse::Button::Right => {
                    self.state.right_mouse_down = false;
                }
                _ => {}
            },
            Event::Mouse(iced_native::mouse::Event::CursorMoved { x, y }) => {
                if layout_bounds.contains(Point::new(x, y)) {
                    let rel_pos = get_relative_position(&layout_bounds, &cursor_position);

                    let index = (rel_pos.x / block_size) as usize
                        + ((rel_pos.y / block_size) as usize * sks::LEVEL_WIDTH);
                    if self.state.left_mouse_down {
                        if let Some(block) = self.active_block.cloned() {
                            messages.push(crate::ui::Message::AddBlock { index, block });
                        }
                    }

                    if self.state.right_mouse_down {
                        messages.push(crate::ui::Message::AddBlock {
                            index,
                            block: sks::Block::Empty,
                        });
                    }
                }
            }
            _ => {}
        }
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
        let layout_bounds_width = layout_bounds.width;

        // one per block + one for bg + one for grid overlay
        let mut primitives = Vec::with_capacity(sks::LEVEL_SIZE + 2);
        let block_size = layout_bounds_width / sks::LEVEL_WIDTH as f32;

        primitives.push(Primitive::Image {
            handle: self.background_image.clone(),
            bounds: layout.bounds(),
        });

        for (i, block) in self.level.get_level_data().iter().enumerate() {
            if !block.is_empty() {
                let x = ((i % sks::LEVEL_WIDTH) as f32 * block_size) + layout_bounds.x;
                let y = ((i / sks::LEVEL_WIDTH) as f32 * block_size) + layout_bounds.y;
                let bounds = Rectangle {
                    x,
                    y,
                    width: block_size,
                    height: block_size,
                };

                let handle = self.iced_block_map.get(block.clone());
                primitives.push(Primitive::Image { handle, bounds });
            }
        }

        let grid_thickness = 2;
        if self.grid {
            let mut grid_primitives = Vec::with_capacity(sks::LEVEL_SIZE);
            for i in 0..sks::LEVEL_SIZE {
                let x = ((i % sks::LEVEL_WIDTH) as f32 * block_size) + layout_bounds.x;
                let y = ((i / sks::LEVEL_WIDTH) as f32 * block_size) + layout_bounds.y;
                let bounds = Rectangle {
                    x,
                    y,
                    width: block_size,
                    height: block_size,
                };

                grid_primitives.push(Primitive::Quad {
                    bounds,
                    background: Background::Color(Color::TRANSPARENT),
                    border_radius: 0,
                    border_width: grid_thickness,
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

impl<'a, B> Into<Element<'a, crate::ui::Message, Renderer<B>>> for Board<'a>
where
    B: Backend,
{
    fn into(self) -> Element<'a, crate::ui::Message, Renderer<B>> {
        Element::new(self)
    }
}
