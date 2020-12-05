use crate::ui::DarkTheme;
use iced::Length;

#[derive(Debug, Clone)]
pub enum NoteModalMessage {
    Open,
    Close,

    InputChanged(String),
    Submit,
}

pub struct NoteModal {
    close_button_state: iced::widget::button::State,
    text_input_state: iced::widget::text_input::State,
    text_input_content: String,
}

impl NoteModal {
    pub fn new() -> Self {
        NoteModal {
            close_button_state: iced::widget::button::State::new(),
            text_input_state: iced::widget::text_input::State::new(),
            text_input_content: String::new(),
        }
    }

    pub fn take_content(&mut self) -> String {
        std::mem::take(&mut self.text_input_content)
    }

    pub fn view(&mut self) -> iced::Element<NoteModalMessage> {
        let default_padding = 10;

        let title = iced::Text::new("Note Content")
            .size(70)
            .horizontal_alignment(iced::HorizontalAlignment::Center);

        let input = iced::TextInput::new(
            &mut self.text_input_state,
            "note content...",
            &self.text_input_content,
            NoteModalMessage::InputChanged,
        )
        .on_submit(NoteModalMessage::Submit)
        .size(50)
        .padding(default_padding);

        let exit_button = iced_native::Button::new(
            &mut self.close_button_state,
            iced::Text::new("Exit").size(30),
        )
        .padding(default_padding)
        .style(DarkTheme::primary())
        .on_press(NoteModalMessage::Close);

        let main_content = iced::Container::new(
            iced::Column::new()
                .push(title)
                .push(input)
                .push(iced_native::Space::new(Length::Fill, Length::Fill))
                .push(exit_button)
                .align_items(iced::Align::Center)
                .spacing(default_padding)
                .width(Length::Fill),
        )
        .padding(default_padding)
        .style(DarkTheme::primary())
        .center_x()
        .width(Length::Fill)
        .height(Length::Fill);

        iced::Container::new(main_content)
            .padding(default_padding)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }

    pub fn update(&mut self, message: NoteModalMessage) {
        match message {
            NoteModalMessage::Open => {
                self.text_input_state = iced::widget::text_input::State::new();
                self.text_input_content.clear();
            }
            NoteModalMessage::InputChanged(content) => {
                self.text_input_content = content;
            }
            _ => {}
        }
    }
}

impl Default for NoteModal {
    fn default() -> Self {
        Self::new()
    }
}
