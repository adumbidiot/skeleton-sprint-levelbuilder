use crate::ui::get_relative_position;
use iced_core::{
    Background,
    Color,
    Length,
    Point,
    Rectangle,
    Size,
};
use iced_graphics::{
    Backend,
    Defaults,
    Primitive,
    Renderer,
};
use iced_native::{
    keyboard,
    keyboard::KeyCode,
    layout,
    mouse,
    Clipboard,
    Element,
    Event,
    Layout,
    Widget,
};
use sks::block::Direction as SksDirection;
use std::hash::{
    Hash,
    Hasher,
};

// TODO: Maybe this could be user configurable some day?
const TOOLBAR_BLOCKS: &[sks::Block] = &[
    sks::Block::Block,
    sks::Block::Empty,
    sks::Block::PipeIn,
    sks::Block::PipeOut,
    sks::Block::PipePhase,
    sks::Block::PipeSolid,
    sks::Block::ToggleBlock { solid: true },
    sks::Block::ToggleBlock { solid: false },
    sks::Block::Lock,
    sks::Block::Key,
    sks::Block::OneWayWall {
        direction: SksDirection::Up,
    },
    sks::Block::OneWayWall {
        direction: SksDirection::Down,
    },
    sks::Block::OneWayWall {
        direction: SksDirection::Left,
    },
    sks::Block::OneWayWall {
        direction: SksDirection::Right,
    },
    sks::Block::Switch,
    sks::Block::SwitchCeiling,
    sks::Block::Player,
    sks::Block::Exit,
    sks::Block::Torch,
    sks::Block::Scaffold,
    sks::Block::PowerUpBurrow,
    sks::Block::PowerUpRecall,
    sks::Block::Note {
        text: String::new(),
    },
    sks::Block::Wire,
    sks::Block::SecretExit,
];

fn strip_block(block: &sks::Block) -> sks::Block {
    match block {
        sks::Block::Note { .. } => sks::Block::Note {
            text: String::new(),
        },
        block => block.clone(),
    }
}

#[derive(Debug)]
pub struct State {
    selected: Option<usize>,
    block_size: f32,
}

impl State {
    pub fn new() -> State {
        State {
            selected: None,
            block_size: 40.0,
        }
    }

    pub fn select_block(&mut self, block: Option<&sks::Block>) {
        match block.map(strip_block) {
            Some(block) => {
                if let Some(index) = TOOLBAR_BLOCKS.iter().position(|el| el == &block) {
                    self.selected = Some(index);
                }
            }
            None => {
                self.selected = None;
            }
        }
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.selected.hash(state);
        self.block_size.to_ne_bytes().hash(state);
    }
}

pub struct ToolBar<'a> {
    iced_block_map: &'a crate::IcedBlockMap,
    state: &'a mut State,
    iced_trash_bin_image: &'a iced_native::image::Handle,
}

impl<'a> ToolBar<'a> {
    pub fn new(
        iced_block_map: &'a crate::IcedBlockMap,
        state: &'a mut State,
        iced_trash_bin_image: &'a iced_native::image::Handle,
    ) -> Self {
        Self {
            iced_block_map,
            state,
            iced_trash_bin_image,
        }
    }
}

impl<'a, B> Widget<crate::ui::Message, Renderer<B>> for ToolBar<'a>
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
        let max_width = self.state.block_size * 2.0;
        let size = limits
            .max_width(max_width as u32)
            .resolve(Size::new(max_width, limits.max().height));
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
    ) -> iced_native::event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let layout_bounds = layout.bounds();

                if layout_bounds.contains(cursor_position) {
                    let rel_pos = get_relative_position(&layout_bounds, &cursor_position);
                    let index_x = (rel_pos.x / self.state.block_size) as usize;
                    let click_index =
                        index_x + ((rel_pos.y / self.state.block_size as f32) as usize * 2);

                    if let Some(block_ref) = TOOLBAR_BLOCKS.get(click_index) {
                        if Some(click_index) == self.state.selected {
                            self.state.selected = None;
                            messages.push(crate::ui::Message::ChangeActiveBlock { block: None });
                        } else if block_ref.is_note() {
                            // self.state.selected = Some(click_index);
                            messages.push(crate::ui::Message::OpenNoteModal);
                        } else {
                            self.state.selected = Some(click_index);
                            messages.push(crate::ui::Message::ChangeActiveBlock {
                                block: Some(block_ref.clone()),
                            });
                        }

                        return iced_native::event::Status::Captured;
                    }
                }

                iced_native::event::Status::Ignored
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                if self
                    .state
                    .selected
                    .and_then(|index| TOOLBAR_BLOCKS.get(index))
                    .map_or(false, sks::Block::is_directional)
                {
                    let maybe_new_selected = match key_code {
                        KeyCode::Up | KeyCode::W => Some(sks::Block::OneWayWall {
                            direction: SksDirection::Up,
                        }),
                        KeyCode::Left | KeyCode::A => Some(sks::Block::OneWayWall {
                            direction: SksDirection::Left,
                        }),
                        KeyCode::Down | KeyCode::S => Some(sks::Block::OneWayWall {
                            direction: SksDirection::Down,
                        }),
                        KeyCode::Right | KeyCode::D => Some(sks::Block::OneWayWall {
                            direction: SksDirection::Right,
                        }),
                        _ => None,
                    };

                    let maybe_new_selected = maybe_new_selected
                        .and_then(|block| TOOLBAR_BLOCKS.iter().position(|el| el == &block));

                    if let Some(index) = maybe_new_selected {
                        if Some(index) != self.state.selected {
                            messages.push(crate::ui::Message::ChangeActiveBlock {
                                block: Some(TOOLBAR_BLOCKS[index].clone()),
                            });
                            self.state.selected = Some(index);

                            return iced_native::event::Status::Captured;
                        }
                    }

                    iced_native::event::Status::Ignored
                } else {
                    iced_native::event::Status::Ignored
                }
            }
            Event::Window(iced_native::window::Event::Resized { width, height }) => {
                if width != 0 && height != 0 {
                    // self.state.block_size = width as f32 / sks::LEVEL_WIDTH as f32;
                    self.state.block_size = 40.0;
                }

                iced_native::event::Status::Ignored
            }
            _ => iced_native::event::Status::Ignored,
        }
    }

    fn hash_layout(&self, state: &mut iced_native::Hasher) {
        self.state.hash(state);
    }

    fn draw(
        &self,
        _renderer: &mut Renderer<B>,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        let layout_bounds = layout.bounds();

        // One per toolbar block and one for the selected outline
        let mut primitives = Vec::with_capacity(TOOLBAR_BLOCKS.len() + 1);

        for (i, block) in TOOLBAR_BLOCKS.iter().enumerate() {
            let x = layout_bounds.x + ((i % 2) as f32 * self.state.block_size);
            let y = layout_bounds.y + ((i / 2) as f32 * self.state.block_size);
            // Add boundary
            let border_size = 8.0;
            let bounds = Rectangle {
                x: x + (border_size / 2.0),
                y: y + (border_size / 2.0),
                width: (self.state.block_size - border_size).max(0.0),
                height: (self.state.block_size - border_size).max(0.0),
            };

            let start = bounds.position();
            let end = start + bounds.size().into();
            if layout_bounds.contains(start) && layout_bounds.contains(end) {
                let handle = if !block.is_empty() {
                    self.iced_block_map.get(block.clone())
                } else {
                    self.iced_trash_bin_image.clone()
                };
                primitives.push(Primitive::Image { handle, bounds });
            }
        }

        if let Some(index) = self.state.selected {
            let x = layout_bounds.x + ((index % 2) as f32 * self.state.block_size);
            let y = layout_bounds.y + ((index / 2) as f32 * self.state.block_size);

            let bounds = Rectangle {
                x,
                y,
                width: self.state.block_size,
                height: self.state.block_size,
            };

            primitives.push(Primitive::Quad {
                bounds,
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0.0,
                border_width: 4.0,
                border_color: Color::from_rgb8(255, 0, 0),
            });
        }

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }
}

impl<'a, B> Into<Element<'a, crate::ui::Message, Renderer<B>>> for ToolBar<'a>
where
    B: Backend,
{
    fn into(self) -> Element<'a, crate::ui::Message, Renderer<B>> {
        Element::new(self)
    }
}
