use crate::{actions::WORKSPACE_CONTEXT, file_tree::FileTree, prelude::*};
use gpui_kit::{
    base::dock::{DockArea, DockLayout},
    component::dock::{DockSkin, panel_handle},
};
use std::{path::PathBuf, rc::Rc};

pub struct Workspace {
    pub root: PathBuf,
    pub name: SharedString,
    dock_area: Entity<DockArea>,
    _dock_skin: Rc<DockSkin>,
    _file_tree: Entity<FileTree>,
    focus: FocusHandle,
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
            _file_tree,
            focus: cx.focus_handle(),
        }
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
