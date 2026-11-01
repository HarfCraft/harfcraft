use crate::{
    actions::WORKSPACE_CONTEXT,
    buffer::{Buffer, BufferEvent},
    file_tree::{FileTree, FileTreeEvent},
    prelude::*,
    preview::Preview,
};
use gpui_kit::{
    base::dock::{DockArea, DockLayout},
    component::{
        dock::{DockSkin, panel_handle},
        status_bar::StatusBar,
    },
};
use std::{collections::HashMap, path::PathBuf, rc::Rc};

pub struct Workspace {
    pub root: PathBuf,
    pub name: SharedString,
    focus: FocusHandle,
    dock_area: Entity<DockArea>,
    _dock_skin: Rc<DockSkin>,
    _file_tree: Entity<FileTree>,
    _preview: Entity<Preview>,
    buffers: HashMap<PathBuf, Entity<Buffer>>,
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
        let _preview = cx.new(|cx| Preview::new(root.clone(), window, cx));
        let (dock_area, _dock_skin) = DockSkin::dock_area("workspace-dock", None, window, cx);
        // _dock_skin.set_panel_style(gpui_kit::component::dock::PanelStyle::TabBar, cx);
        dock_area.update(cx, |dock_area, cx| {
            dock_area.set_dock(
                gpui_kit::base::dock::DockPlacement::Left,
                DockLayout::tabs().panel_view(panel_handle(_file_tree.clone()), cx),
                window,
                cx,
            );
            dock_area.set_dock(
                gpui_kit::base::dock::DockPlacement::Bottom,
                DockLayout::tabs().panel_view(panel_handle(_preview.clone()), cx),
                window,
                cx,
            );
        });
        _dock_skin.set_panel_style(gpui_kit::component::dock::PanelStyle::TabBar, cx);

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
            buffers: HashMap::new(),
            _file_tree,
            _preview,
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
        if let Some(buffer) = self.buffers.get(&path) {
            self.activate_buffer(buffer.clone(), window, cx);
            return;
        }
        let buffer: Entity<Buffer> = cx.new(|cx| Buffer::new(path.clone(), name, window, cx));
        self.dock_area.update(cx, |dock_area, cx| {
            dock_area.add_panel_view(
                panel_handle(buffer.clone()),
                gpui_kit::base::dock::DockPlacement::Center,
                None,
                window,
                cx,
            );
        });
        cx.subscribe_in(
            &buffer,
            window,
            |this, buffer, ev: &BufferEvent, window: &mut Window, cx| match ev {
                BufferEvent::Close(path) => {
                    this.buffers.remove(path.as_path());
                    this.dock_area.update(cx, move |dock_area, cx| {
                        dock_area.remove_panel(buffer.clone(), window, cx);
                    });
                }
            },
        )
        .detach();
        self.buffers.insert(path, buffer);
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
            .child(StatusBar::new().child(format!("{} buffer(s)", self.buffers.len())))
    }
}
