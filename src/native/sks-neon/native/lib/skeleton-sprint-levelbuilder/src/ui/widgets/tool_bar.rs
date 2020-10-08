use crate::ui::get_relative_position;
use iced_core::Background;
use iced_core::Color;
use iced_core::Length;
use iced_core::Point;
use iced_core::Rectangle;
use iced_core::Size;
use iced_graphics::Backend;
use iced_graphics::Defaults;
use iced_graphics::Primitive;
use iced_graphics::Renderer;
use iced_native::keyboard;
use iced_native::keyboard::KeyCode;
use iced_native::layout;
use iced_native::mouse;
use iced_native::Clipboard;
use iced_native::Element;
use iced_native::Event;
use iced_native::Hasher;
use iced_native::Layout;
use iced_native::Widget;
use sks::block::Direction as SksDirection;

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

fn is_directional(block: &sks::Block) -> bool {
    matches!(block, sks::Block::OneWayWall { .. })
}

pub struct State {
    selected: Option<usize>,
}

impl State {
    pub fn new() -> State {
        State { selected: None }
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
        let block_size = crate::WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;
        let max_width = (block_size * 2) as f32;
        let size = limits
            .max_width(max_width as u32)
            .resolve(Size::new(max_width, crate::WINDOW_HEIGHT as f32));
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
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let layout_bounds = layout.bounds();
                let block_size = crate::WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;

                if layout_bounds.contains(cursor_position) {
                    let rel_pos = get_relative_position(&layout_bounds, &cursor_position);
                    let click_index = (rel_pos.x / block_size as f32) as usize
                        + ((rel_pos.y / block_size as f32) as usize * 2);

                    if let Some(block_ref) = TOOLBAR_BLOCKS.get(click_index) {
                        if Some(click_index) == self.state.selected {
                            self.state.selected = None;
                            messages.push(crate::ui::Message::ChangeActiveBlock { block: None });
                        } else {
                            self.state.selected = Some(click_index);
                            messages.push(crate::ui::Message::ChangeActiveBlock {
                                block: Some(block_ref.clone()),
                            });
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                if self
                    .state
                    .selected
                    .and_then(|index| TOOLBAR_BLOCKS.get(index))
                    .map_or(false, is_directional)
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
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        // 0u32.hash(state);
        // self.level.hash(state);
        // self.grid.hash(state);
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
        let block_size = crate::WINDOW_WIDTH / sks::LEVEL_WIDTH as u32;

        // One per toolbar block and one for the selected outline
        let mut primitives = Vec::with_capacity(TOOLBAR_BLOCKS.len() + 1);

        for (i, block) in TOOLBAR_BLOCKS.iter().enumerate() {
            let x = layout_bounds.x + ((i % 2) as f32 * block_size as f32);
            let y = layout_bounds.y + ((i / 2) as f32 * block_size as f32);
            // Add boundary
            let border_size: u8 = 8;
            let bounds = Rectangle {
                x: x + f32::from(border_size / 2),
                y: y + f32::from(border_size / 2),
                width: (block_size - u32::from(border_size)) as f32,
                height: (block_size - u32::from(border_size)) as f32,
            };

            let handle = if !block.is_empty() {
                self.iced_block_map.get(block.clone())
            } else {
                self.iced_trash_bin_image.clone()
            };
            primitives.push(Primitive::Image { handle, bounds });
        }

        if let Some(index) = self.state.selected {
            let x = layout_bounds.x + ((index % 2) as f32 * block_size as f32);
            let y = layout_bounds.y + ((index / 2) as f32 * block_size as f32);

            let bounds = Rectangle {
                x,
                y,
                width: block_size as f32,
                height: block_size as f32,
            };

            primitives.push(Primitive::Quad {
                bounds,
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0,
                border_width: 4,
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
