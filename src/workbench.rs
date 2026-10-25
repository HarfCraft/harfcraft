use crate::prelude::*;
use gpui_kit::component::{TitleBar, status_bar::StatusBar};

pub struct Workbench {
    focus: FocusHandle,
}

impl Focusable for Workbench {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Workbench {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
        }
    }
}

impl Render for Workbench {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .track_focus(&self.focus)
            .size_full()
            .child(TitleBar::new())
            .child(div().size_full().child("Main"))
            .child(StatusBar::new().child("Status"))
    }
}
