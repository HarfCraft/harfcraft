use crate::{
    actions::WORKSPACE_CONTEXT,
    buffer::Buffer,
    file_tree::{FileTree, FileTreeEvent},
    prelude::*,
};
use gpui_kit::{
    base::dock::{DockArea, DockLayout},
    component::dock::{DockSkin, panel_handle},
};
use std::{path::PathBuf, rc::Rc};

pub struct Workspace {
    pub root: PathBuf,
    pub name: SharedString,
    focus: FocusHandle,
    dock_area: Entity<DockArea>,
    _dock_skin: Rc<DockSkin>,
    _file_tree: Entity<FileTree>,
    buffers: Vec<Entity<Buffer>>,
    _subscriptions: Vec<Subscription>,
}

impl Focusable for Workspace {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Workspace {
    pub fn new(root: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _file_tree = cx.new(|cx| FileTree::new(root.clone(), cx));
        let (dock_area, _dock_skin) = DockSkin::dock_area("workspace-dock", None, window, cx);
        // _dock_skin.set_panel_style(gpui_kit::component::dock::PanelStyle::TabBar, cx);
        dock_area.update(cx, |dock_area, cx| {
            dock_area.set_dock(
                gpui_kit::base::dock::DockPlacement::Left,
                DockLayout::tabs().panel_view(panel_handle(_file_tree.clone()), cx),
                window,
                cx,
            );
        });

        Self {
            name: SharedString::from(
                root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or("Unnamed".to_string()),
            ),
            root,
            dock_area,
            _dock_skin,
            focus: cx.focus_handle(),
            _subscriptions: vec![cx.subscribe_in(&_file_tree, window, Self::on_file_tree_event)],
            buffers: Vec::new(),
            _file_tree,
        }
    }

    fn on_file_tree_event(
        &mut self,
        _: &Entity<FileTree>,
        event: &FileTreeEvent,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        match event {
            FileTreeEvent::OpenEntry(path, name) => {
                self.new_buffer(path.clone(), name.clone(), window, cx);
            }
        }
    }

    fn new_buffer(
        &mut self,
        path: PathBuf,
        name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(buffer) = self.buffers.iter().find(|b| b.read(cx).path == path) {
            self.activate_buffer(buffer.clone(), window, cx);
            return;
        }
        let buffer: Entity<Buffer> = cx.new(|cx| Buffer::new(path, name, window, cx));
        self.dock_area.update(cx, |dock_area, cx| {
            dock_area.add_panel_view(
                panel_handle(buffer.clone()),
                gpui_kit::base::dock::DockPlacement::Center,
                None,
                window,
                cx,
            );
        });
        self.buffers.push(buffer);
    }

    fn activate_buffer(
        &mut self,
        buffer: Entity<Buffer>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dock_area.update(cx, |dock_area, cx| {
            dock_area.select_panel(panel_handle(buffer).panel_id(cx), window, cx);
        });
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(WORKSPACE_CONTEXT)
            .track_focus(&self.focus)
            .size_full()
            .child(self.dock_area.clone())
    }
}
