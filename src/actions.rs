use gpui_kit::{App, KeyBinding, actions};

pub const WORKBENCH_CONTEXT: &'static str = "workbench";
actions!(WORKBENCH_CONTEXT, [OpenWorkspace]);

pub const WORKSPACE_CONTEXT: &'static str = "workspace";
actions!(WORKSPACE_CONTEXT, []);

pub const FILE_TREE_CONTEXT: &'static str = "file-tree";
actions!(
    FILE_TREE_CONTEXT,
    [PreviousFileTreeItem, NextFileTreeItem, ActivateFileTreeItem]
);

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "ctrl-o",
        OpenWorkspace,
        Some(WORKBENCH_CONTEXT),
    )]);
    init_file_tree(cx);
}

fn init_file_tree(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", PreviousFileTreeItem, Some(FILE_TREE_CONTEXT)),
        KeyBinding::new("down", NextFileTreeItem, Some(FILE_TREE_CONTEXT)),
        KeyBinding::new("enter", ActivateFileTreeItem, Some(FILE_TREE_CONTEXT)),
    ]);
}
