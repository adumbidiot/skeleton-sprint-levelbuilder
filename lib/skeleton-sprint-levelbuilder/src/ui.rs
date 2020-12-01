mod widgets;

use iced_core::{
    Point,
    Rectangle,
};

/// Assumes it CAN be translated infallibly. TODO: Do i make this return an option?
pub fn get_relative_position(bounds: &Rectangle, pos: &Point) -> Point {
    Point::new(pos.x - bounds.x, pos.y - bounds.y)
}

#[derive(Debug)]
pub enum AppState {
    Builder,
    NoteModal,
}

impl AppState {
    pub fn new() -> Self {
        AppState::Builder
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddBlock { index: usize, block: sks::Block },
    ImportLevel { level: crate::Level },
    SetDark { dark: bool },
    SetGrid { grid: bool },
    ChangeActiveBlock { block: Option<sks::Block> },
    OpenNoteModal,

    NoteModalInputChanged(String),
    NoteModalSubmit { is_success: bool },

    Nop,
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

    app_state: AppState,

    note_modal_close_button_state: iced_native::widget::button::State,
    note_modal_text_input_state: iced_native::widget::text_input::State,
    note_modal_text_input_content: String,
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

            app_state: AppState::Builder,

            note_modal_close_button_state: iced_native::widget::button::State::new(),
            note_modal_text_input_state: iced_native::widget::text_input::State::new(),
            note_modal_text_input_content: String::new(),
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
            Message::OpenNoteModal => {
                self.note_modal_text_input_state = iced_native::widget::text_input::State::new();
                self.note_modal_text_input_content.clear();
                self.app_state = AppState::NoteModal;
            }
            Message::NoteModalInputChanged(content) => {
                self.note_modal_text_input_content = content;
            }
            Message::NoteModalSubmit { is_success } => {
                if is_success {
                    let text = std::mem::take(&mut self.note_modal_text_input_content);
                    self.active_block = Some(sks::Block::Note { text });
                    self.tool_bar_state.select_block(self.active_block.as_ref());
                }

                self.app_state = AppState::Builder;
            }
            Message::Nop => {}
        }

        iced_native::Command::none()
    }

    fn view(&mut self) -> iced_native::Element<Self::Message, Self::Renderer> {
        use self::widgets::{
            Board,
            ToolBar,
        };
        use iced_core::{
            Color,
            Length,
        };
        use iced_graphics::{
            container,
            Text,
        };

        match self.app_state {
            AppState::Builder => {
                let main_content = iced_native::Row::new()
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
                    .padding(20);

                iced_native::widget::Container::new(
                    iced_native::Column::new()
                        .push(
                            iced_native::Container::new(
                                iced_native::Container::new(
                                    iced_native::Row::new()
                                        .push(
                                            Text::new("SS")
                                                .size(80)
                                                .horizontal_alignment(
                                                    iced_core::HorizontalAlignment::Center,
                                                )
                                                .vertical_alignment(
                                                    iced_core::VerticalAlignment::Center,
                                                ),
                                        )
                                        .spacing(20),
                                )
                                .padding(20),
                            )
                            .height(Length::Units(100))
                            .width(Length::Fill)
                            .style(DarkerTheme),
                        )
                        .push(main_content),
                )
                .into()
            }
            AppState::NoteModal => {
                let main_content = iced_native::Container::new(
                    iced_native::Column::new()
                        .push(
                            Text::new("Note Content")
                                .size(70)
                                .horizontal_alignment(iced_core::HorizontalAlignment::Center),
                        )
                        .push(
                            iced_native::TextInput::new(
                                &mut self.note_modal_text_input_state,
                                "note content...",
                                &self.note_modal_text_input_content,
                                Message::NoteModalInputChanged,
                            )
                            .on_submit(Message::NoteModalSubmit { is_success: true })
                            .size(50)
                            .padding(20),
                        )
                        .push(iced_native::Space::new(Length::Fill, Length::Fill))
                        .push(
                            iced_native::Button::new(
                                &mut self.note_modal_close_button_state,
                                Text::new("Exit").size(70),
                            )
                            .padding(20)
                            .on_press(Message::NoteModalSubmit { is_success: false }),
                        )
                        .align_items(iced_core::Align::Center)
                        .spacing(20)
                        .width(Length::Fill),
                )
                .padding(20)
                .style(Theme)
                .center_x()
                .width(Length::Fill)
                .height(Length::Fill);

                iced_native::Container::new(main_content)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .into()
            }
        }
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

pub struct DarkerTheme;

impl From<DarkerTheme> for Box<dyn iced_graphics::container::StyleSheet> {
    fn from(theme: DarkerTheme) -> Self {
        DarkerContainer.into()
    }
}

pub struct DarkerContainer;

impl iced_graphics::container::StyleSheet for DarkerContainer {
    fn style(&self) -> iced_graphics::container::Style {
        use iced_core::Color;
        use iced_graphics::container;

        container::Style {
            background: Color::from_rgb8(0x30, 0x30, 0x30).into(),
            text_color: Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}
