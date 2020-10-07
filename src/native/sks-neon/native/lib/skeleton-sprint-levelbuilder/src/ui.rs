mod widgets;

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

    board_state: widgets::board::State,
}

impl UiApp {
    pub fn new(
        iced_block_map: crate::IcedBlockMap,
        iced_background_image: iced_native::image::Handle,
    ) -> Self {
        Self {
            level: crate::Level::new(),
            active_block: None,

            grid: true,

            iced_block_map,
            iced_background_image,

            board_state: widgets::board::State::new(),
        }
    }
}

impl iced_native::Program for UiApp {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> iced_native::Command<Message> {
        match message {
            Message::AddBlock { index, block } => {
                let success = self.level.add_block(index, block).is_none();
                assert!(success);
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
        iced_native::Row::new()
            .push(
                iced_native::widget::Container::new(
                    self::widgets::Board::new(
                        &self.level,
                        &self.iced_background_image,
                        &self.iced_block_map,
                        &mut self.board_state,
                    )
                    .grid(self.grid)
                    .active_block(self.active_block.as_ref()),
                )
                .width(iced_core::Length::FillPortion(4)),
            )
            /*
            .push(
                iced_native::widget::Text::new("Test")
                    .size(50)
                    .width(iced_core::Length::FillPortion(1)),
            )
            .spacing(20)*/
            .into()
    }
}
