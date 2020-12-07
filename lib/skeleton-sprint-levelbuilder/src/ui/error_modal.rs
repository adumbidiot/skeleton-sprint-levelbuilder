use crate::{
    ui::DarkTheme,
    AppError,
};
use iced::Length;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ErrorModalMessage {
    Open(Arc<AppError>),
    Close,
}

pub struct ErrorModal {
    error: Option<Arc<AppError>>,
    ok_button_state: iced::widget::button::State,
}

impl ErrorModal {
    pub fn new() -> Self {
        ErrorModal {
            error: None,
            ok_button_state: Default::default(),
        }
    }

    pub fn view(&mut self) -> iced::Element<ErrorModalMessage> {
        let default_padding = 10;

        let title = iced::Text::new("Error")
            .size(70)
            .horizontal_alignment(iced::HorizontalAlignment::Center);

        let error_msg = iced::Text::new(
            self.error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Error was not specified".into()),
        )
        .size(70)
        .horizontal_alignment(iced::HorizontalAlignment::Center);

        let ok_button =
            iced::Button::new(&mut self.ok_button_state, iced::Text::new("Ok").size(30))
                .padding(default_padding)
                .style(DarkTheme::primary())
                .on_press(ErrorModalMessage::Close);

        let main_content = iced_native::Container::new(
            iced_native::Column::new()
                .push(title)
                .push(iced::Space::new(Length::Fill, Length::Fill))
                .push(error_msg)
                .push(iced::Space::new(Length::Fill, Length::Fill))
                .push(ok_button)
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

    pub fn update(&mut self, msg: ErrorModalMessage) {
        match msg {
            ErrorModalMessage::Open(error) => self.error = Some(error),
            ErrorModalMessage::Close => self.error = None,
        }
    }
}

impl Default for ErrorModal {
    fn default() -> Self {
        Self::new()
    }
}
