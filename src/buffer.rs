use crate::{
    actions::{BUFFER_CONTEXT, CloseBuffer, SaveBuffer},
    prelude::*,
};
use gpui_kit::{
    base::input::{EditorState, InputEvent},
    component::{alert::Alert, input::Editor},
};
use std::{fs, path::PathBuf};

pub struct Buffer {
    pub path: PathBuf,
    pub name: SharedString,
    status: BufferStatus,
    editor: Entity<EditorState>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

pub enum BufferStatus {
    Loading,
    Synced,
    Dirty,
    Saved,
    Conflict(String),
    Error(SharedString),
}

impl BufferStatus {
    pub fn error(&self) -> Option<SharedString> {
        if let BufferStatus::Error(err) = self {
            Some(err.clone())
        } else {
            None
        }
    }
    pub fn is_readonly(&self) -> bool {
        match self {
            BufferStatus::Loading | BufferStatus::Error(_) => true,
            _ => false,
        }
    }
    pub fn icon(&self) -> IconName {
        match self {
            BufferStatus::Loading => IconName::Loader,
            BufferStatus::Synced | BufferStatus::Saved => IconName::File,
            BufferStatus::Dirty => IconName::Dot,
            BufferStatus::Conflict(_) => IconName::RefreshCwOff,
            BufferStatus::Error(_) => IconName::TriangleAlert,
        }
    }
}

pub enum BufferEvent {
    Close(PathBuf),
}

impl EventEmitter<BufferEvent> for Buffer {}

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
    fn on_removed(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(BufferEvent::Close(self.path.clone()));
    }
}

impl Panel for Buffer {
    fn title(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_1()
            .child(Icon::new(self.status.icon()))
            .child(self.name.clone())
            .child(
                Button::new(format!("close-buffer-{}", self.name.clone()))
                    .mr_neg_2()
                    .small()
                    .ghost()
                    .icon(IconName::Close)
                    .tooltip_with_action("Close", &CloseBuffer, Some(BUFFER_CONTEXT))
                    .on_click(
                        cx.listener(|this, _, _, cx| {
                            cx.emit(BufferEvent::Close(this.path.clone()))
                        }),
                    ),
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
        let editor = cx.new(|cx| EditorState::new(window, cx));
        let file_path = path.clone();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { fs::read_to_string(file_path) })
                .await;
            match result {
                Ok(contents) => {
                    _ = this.update_in(cx, |buffer, window, cx| {
                        buffer.editor.update(cx, |editor, cx| {
                            editor.set_value(contents, window, cx);
                        });
                        buffer.status = BufferStatus::Synced;
                        cx.notify();
                    });
                }
                Err(err) => {
                    _ = this.update(cx, |buffer, cx| {
                        buffer.status = BufferStatus::Error(err.to_string().into());
                        cx.notify();
                    });
                }
            }
        })
        .detach();
        Self {
            path,
            name,
            status: BufferStatus::Loading,
            _subscriptions: vec![cx.subscribe(&editor, |this, _, ev: &InputEvent, cx| {
                if let InputEvent::Change = ev {
                    this.status = BufferStatus::Dirty;
                    cx.notify();
                }
            })],
            editor,
            focus: cx.focus_handle(),
        }
    }
    pub fn save_buffer(&mut self, _: &SaveBuffer, _: &mut Window, cx: &mut Context<Self>) {
        let contents = self.editor.read(cx).value().to_string();
        match fs::write(&self.path, contents) {
            Ok(_) => {
                self.status = BufferStatus::Saved;
                cx.notify();
            }
            Err(err) => {
                self.status = BufferStatus::Error(err.to_string().into());
                cx.notify();
            }
        }
    }

    pub fn close_buffer(&mut self, _: &CloseBuffer, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(BufferEvent::Close(self.path.clone()));
    }
}

impl Render for Buffer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .track_focus(&self.focus)
            .key_context(BUFFER_CONTEXT)
            .on_action(cx.listener(Self::save_buffer))
            .on_action(cx.listener(Self::close_buffer))
            .size_full()
            .when_some(self.status.error(), |this, err| {
                this.child(Alert::error("buffer-error", err).rounded_none())
            })
            .child(
                Editor::new(&self.editor)
                    .readonly(self.status.is_readonly())
                    .bordered(false)
                    .size_full(),
            )
    }
}
