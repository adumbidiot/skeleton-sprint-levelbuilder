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

impl From<DarkTheme> for Box<dyn iced_graphics::button::StyleSheet> {
    fn from(theme: DarkTheme) -> Self {
        let style = match theme.0 {
            ColorType::Primary => ButtonStyle(iced::Color::from_rgb8(0xFF, 0x00, 0x00)),
            ColorType::Secondary => ButtonStyle(iced::Color::from_rgb8(0x30, 0x30, 0x30)),
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

pub struct ButtonStyle(iced::Color);

impl iced_graphics::button::StyleSheet for ButtonStyle {
    fn active(&self) -> iced_graphics::button::Style {
        iced_graphics::button::Style {
            background: self.0.into(),
            border_radius: 20.0,
            ..iced::button::Style::default()
        }
    }

    fn pressed(&self) -> iced_graphics::button::Style {
        iced_graphics::button::Style {
            background: iced::Color {
                r: (self.0.r - 0.5).max(0.0),
                g: (self.0.g - 0.5).max(0.0),
                b: (self.0.b - 0.5).max(0.0),
                a: self.0.a,
            }
            .into(),
            border_radius: 20.0,
            ..iced::button::Style::default()
        }
    }
}
