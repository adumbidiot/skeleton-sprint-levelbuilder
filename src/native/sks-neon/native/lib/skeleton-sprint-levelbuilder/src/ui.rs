mod widgets;

use iced_core::Point;
use iced_core::Rectangle;

/// Assumes it CAN be translated infallibly. TODO: Do i make this return an option?
pub fn get_relative_position(bounds: &Rectangle, pos: &Point) -> Point {
    Point::new(pos.x - bounds.x, pos.y - bounds.y)
}

#[derive(Debug)]
pub enum Message {
    AddBlock { index: usize, block: sks::Block },
    ImportLevel { level: crate::Level },
    SetDark { dark: bool },
    SetGrid { grid: bool },
    ChangeActiveBlock { block: Option<sks::Block> },
}

pub struct UiApp {
    pub level: crate::Level,
    pub active_block: Option<sks::Block>,

    grid: bool,

    iced_block_map: crate::IcedBlockMap,
    iced_background_image: iced_native::image::Handle,
    iced_trash_bin_image: iced_native::image::Handle,

    board_state: widgets::board::State,
    tool_bar_state: widgets::tool_bar::State,
}

impl UiApp {
    pub fn new(
        iced_block_map: crate::IcedBlockMap,
        iced_background_image: iced_native::image::Handle,
        iced_trash_bin_image: iced_native::image::Handle,
    ) -> Self {
        Self {
            level: crate::Level::new(),
            active_block: None,

            grid: true,

            iced_block_map,
            iced_background_image,
            iced_trash_bin_image,

            board_state: widgets::board::State::new(),
            tool_bar_state: widgets::tool_bar::State::new(),
        }
    }
}

impl iced_native::Program for UiApp {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> iced_native::Command<Message> {
        match message {
            Message::AddBlock { index, block } => {
                assert!(self.level.add_block(index, block).is_none());
            }
            Message::ImportLevel { level } => {
                self.level = level;
            }
            Message::SetDark { dark } => {
                self.level.set_dark(dark);
            }
            Message::SetGrid { grid } => {
                self.grid = grid;
            }
            Message::ChangeActiveBlock { block } => {
                self.active_block = block;
            }
        }

        iced_native::Command::none()
    }

    fn view(&mut self) -> iced_native::Element<Self::Message, Self::Renderer> {
        use self::widgets::Board;
        use self::widgets::ToolBar;
        use iced_core::Color;
        use iced_core::Length;
        use iced_graphics::container;

        iced_native::Row::new()
            .push(
                iced_native::widget::Container::new(
                    Board::new(
                        &self.level,
                        &self.iced_background_image,
                        &self.iced_block_map,
                        &mut self.board_state,
                    )
                    .grid(self.grid)
                    .active_block(self.active_block.as_ref()),
                )
                .padding(20)
                .style(Theme)
                .center_x()
                .center_y()
                .height(Length::Fill)
                .width(Length::FillPortion(4)),
            )
            .push(
                iced_native::Container::new(ToolBar::new(
                    &self.iced_block_map,
                    &mut self.tool_bar_state,
                    &self.iced_trash_bin_image,
                ))
                .style(Theme),
            )
            .spacing(20)
            .into()
    }
}

// Kinda-Hack for: https://github.com/hecrj/iced/issues/476. *sigh*
pub struct Theme;

impl From<Theme> for Box<dyn iced_graphics::container::StyleSheet> {
    fn from(theme: Theme) -> Self {
        Container.into()
    }
}

pub struct Container;

impl iced_graphics::container::StyleSheet for Container {
    fn style(&self) -> iced_graphics::container::Style {
        use iced_core::Color;
        use iced_graphics::container;

        container::Style {
            background: Color::from_rgb8(0x77, 0x77, 0x77).into(),
            text_color: Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}
