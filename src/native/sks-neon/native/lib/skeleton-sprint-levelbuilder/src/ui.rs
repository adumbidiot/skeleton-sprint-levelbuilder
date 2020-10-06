mod widgets;

#[derive(Debug)]
pub enum Message {
    AddBlock { index: usize, block: sks::Block },
    ImportLevel { level: crate::Level },
    SetDark { dark: bool },
    SetGrid { grid: bool },
}

pub struct UiApp {
    pub level: crate::Level,
    grid: bool,

    iced_block_map: crate::IcedBlockMap,
    iced_background_image: iced_native::image::Handle,
}

impl UiApp {
    pub fn new(
        iced_block_map: crate::IcedBlockMap,
        iced_background_image: iced_native::image::Handle,
    ) -> Self {
        Self {
            level: crate::Level::new(),
            grid: true,

            iced_block_map,
            iced_background_image,
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
        }

        iced_native::Command::none()
    }

    fn view(&mut self) -> iced_native::Element<Self::Message, Self::Renderer> {
        self::widgets::Board::new(
            &self.level,
            &self.iced_background_image,
            &self.iced_block_map,
        )
        .grid(self.grid)
        .into()
    }
}
