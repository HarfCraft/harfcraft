use gpui_kit::{App, KeyBinding, actions};

pub const WORKBENCH_CONTEXT: &'static str = "workbench";
actions!(WORKBENCH_CONTEXT, [OpenWorkspace]);

pub const WORKSPACE_CONTEXT: &'static str = "workspace";
actions!(WORKSPACE_CONTEXT, []);

pub const FILE_TREE_CONTEXT: &'static str = "file-tree";
actions!(FILE_TREE_CONTEXT, [OpenFileTreeItem]);

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "ctrl-o",
        OpenWorkspace,
        Some(WORKBENCH_CONTEXT),
    )]);
}
