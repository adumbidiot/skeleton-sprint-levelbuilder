use iced::Application;
use iced_native::program::Program;

pub struct AppWrap(skeleton_sprint_levelbuilder::App);

impl AppWrap {
    pub fn new() -> Self {
        AppWrap(skeleton_sprint_levelbuilder::App::new().unwrap())
    }
}

impl Default for AppWrap {
    fn default() -> Self {
        Self::new()
    }
}

// This is nothing but a test area, UB is fine for now
impl Application for AppWrap {
    type Executor = iced::executor::Default;
    type Message = skeleton_sprint_levelbuilder::ui::Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (Self::new(), iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Skeleton Sprint Levelbuilder")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        #[allow(clippy::cast_ref_to_mut)]
        unsafe {
            &mut *(self.0.iced_state.program() as *const skeleton_sprint_levelbuilder::ui::UiApp
                as *mut skeleton_sprint_levelbuilder::ui::UiApp)
        }
        .update(message)
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        #[allow(clippy::cast_ref_to_mut)]
        unsafe {
            &mut *(self.0.iced_state.program() as *const skeleton_sprint_levelbuilder::ui::UiApp
                as *mut skeleton_sprint_levelbuilder::ui::UiApp)
        }
        .view()
    }
}

fn main() {
    AppWrap::run(Default::default()).unwrap();
}
