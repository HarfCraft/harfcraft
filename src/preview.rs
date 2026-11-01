use crate::prelude::*;
use gpui_kit::{base::input::TextareaState, component::input::Textarea};
use std::path::PathBuf;

pub struct Preview {
    pub root: PathBuf,
    text_state: Entity<TextareaState>,
    focus: FocusHandle,
}

impl Preview {
    pub fn new(root: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            root,
            text_state: cx.new(|cx| TextareaState::new(window, cx)),
            focus: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for Preview {}

impl Focusable for Preview {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl BasePanel for Preview {
    fn panel_name(&self) -> &'static str {
        "preview"
    }
    fn closable(&self, _cx: &App) -> bool {
        false
    }
    fn zoomable(&self, _cx: &App) -> bool {
        true
    }
}

impl Panel for Preview {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Preview"
    }
    fn zoom_control(&self, _cx: &App) -> Option<gpui_kit::component::dock::PanelControl> {
        Some(gpui_kit::component::dock::PanelControl::Toolbar)
    }
    fn title_suffix(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        Some(
            Button::new("reload-preview")
                .icon(IconName::RefreshCw)
                .xsmall()
                .ghost(),
        )
    }
}

impl Render for Preview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            .child(
                div()
                    .h_20()
                    .w_full()
                    .bg(cx.theme().progress_bar)
                    .child("Preview image here"),
            )
            .child(Textarea::new(&self.text_state))
    }
}
