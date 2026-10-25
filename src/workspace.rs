use crate::prelude::*;
use std::path::PathBuf;

pub struct Workspace {
    pub root: PathBuf,
    pub name: SharedString,
    focus: FocusHandle,
}

impl Focusable for Workspace {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Workspace {
    pub fn new(root: PathBuf, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            name: SharedString::from(
                root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or("Unnamed".to_string()),
            ),
            root,
            focus: cx.focus_handle(),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(self.name.clone())
            .child(self.root.to_string_lossy().to_string())
    }
}
