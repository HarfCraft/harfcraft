use crate::prelude::*;

#[derive(Clone)]
pub struct DraggedWorkspace {
    pub index: usize,
    pub name: SharedString,
}

impl DraggedWorkspace {
    pub fn new(index: usize, name: impl Into<SharedString>) -> Self {
        Self {
            index,
            name: name.into(),
        }
    }
}

impl Render for DraggedWorkspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .text_sm()
            .bg(cx.theme().accent)
            .text_color(cx.theme().accent_foreground)
            .rounded(cx.theme().radius)
            .shadow_md()
            .child(self.name.clone())
    }
}
