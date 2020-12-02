#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ColorType {
    /// Primary Color
    Primary,

    /// Secondary Color
    Secondary,
}

#[derive(Debug)]
pub struct DarkTheme(ColorType);

impl DarkTheme {
    pub fn primary() -> Self {
        Self(ColorType::Primary)
    }

    pub fn secondary() -> Self {
        Self(ColorType::Secondary)
    }
}

impl From<DarkTheme> for Box<dyn iced_graphics::container::StyleSheet> {
    fn from(theme: DarkTheme) -> Self {
        let style = match theme.0 {
            ColorType::Primary => ContainerStyle(iced::Color::from_rgb8(0x77, 0x77, 0x77)),
            ColorType::Secondary => ContainerStyle(iced::Color::from_rgb8(0x30, 0x30, 0x30)),
        };

        Box::new(style)
    }
}

pub struct ContainerStyle(iced::Color);

impl iced_graphics::container::StyleSheet for ContainerStyle {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            background: self.0.into(),
            text_color: iced::Color::WHITE.into(),
            ..iced::container::Style::default()
        }
    }
}
