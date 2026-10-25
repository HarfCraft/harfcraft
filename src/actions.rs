use gpui_kit::{App, KeyBinding, actions};

pub const WORKBENCH_CONTEXT: &'static str = "workbench";

actions!(WORKBENCH_CONTEXT, [OpenWorkspace]);

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "ctrl-o",
        OpenWorkspace,
        Some(WORKBENCH_CONTEXT),
    )]);
}
