use crate::{actions::BUFFER_CONTEXT, prelude::*};
use gpui_kit::{base::input::EditorState, component::input::Editor};
use std::path::PathBuf;

pub struct Buffer {
    pub path: PathBuf,
    pub name: SharedString,
    editor: Entity<EditorState>,
    focus: FocusHandle,
}

impl Focusable for Buffer {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl EventEmitter<PanelEvent> for Buffer {}

impl BasePanel for Buffer {
    fn panel_name(&self) -> &'static str {
        "Buffer"
    }
}

impl Panel for Buffer {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().gap_1().child(self.name.clone()).child(
            Button::new(format!("close-buffer-{}", self.name.clone()))
                .mr_neg_2()
                .small()
                .ghost()
                .icon(IconName::Close),
        )
    }
    fn title_bar(&self, _cx: &App) -> bool {
        true
    }
    fn inner_padding(&self, _cx: &App) -> bool {
        false
    }
}

impl Buffer {
    pub fn new(
        path: PathBuf,
        name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            path,
            name,
            editor: cx.new(|cx| EditorState::new(window, cx)),
            focus: cx.focus_handle(),
        }
    }
}

impl Render for Buffer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus)
            .key_context(BUFFER_CONTEXT)
            .size_full()
            .child(Editor::new(&self.editor).bordered(false).size_full())
    }
}
