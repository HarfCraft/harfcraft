use crate::{actions::FILE_TREE_CONTEXT, prelude::*};
use gpui_kit::{
    UniformListScrollHandle,
    base::ScrollbarAxis,
    component::{list::ListItem, progress::Progress, scroll::ScrollableElement},
    uniform_list,
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

const BATCH_SIZE: usize = 64;

pub struct FileTree {
    pub root: PathBuf,
    pub nodes: HashMap<PathBuf, FileTreeNode>,
    pub expanded_dirs: HashSet<PathBuf>,
    visible_nodes: Vec<VisibleFileNode>,
    focus: FocusHandle,
    scroll_handle: UniformListScrollHandle,
    selected: Option<PathBuf>,
    loading: bool,
}

pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: SharedString,
    pub extension: Option<SharedString>,
    pub children: Vec<PathBuf>,
    pub is_dir: bool,
}

#[derive(Clone)]
pub struct VisibleFileNode {
    pub index: usize,
    pub path: PathBuf,
    pub depth: usize,
    pub name: SharedString,
    pub extension: Option<SharedString>,
    pub is_dir: bool,
    pub is_expanded: bool,
}

impl Focusable for FileTree {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl EventEmitter<PanelEvent> for FileTree {}

impl BasePanel for FileTree {
    fn panel_name(&self) -> &'static str {
        "FileTree"
    }
    fn zoomable(&self, _cx: &App) -> bool {
        false
    }
    fn closable(&self, _cx: &App) -> bool {
        false
    }
}

impl Panel for FileTree {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Icon::new(IconName::Folder)
    }

    fn zoom_control(&self, _cx: &App) -> Option<gpui_kit::component::dock::PanelControl> {
        Some(gpui_kit::component::dock::PanelControl::Toolbar)
    }

    fn inner_padding(&self, _cx: &App) -> bool {
        true
    }

    fn title_bar(&self, _cx: &App) -> bool {
        false
    }
}

impl FileTree {
    pub fn new(root: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut file_tree = Self {
            loading: true,
            nodes: HashMap::new(),
            expanded_dirs: HashSet::from([root.clone()]),
            visible_nodes: Vec::new(),
            selected: None,
            root,
            focus: cx.focus_handle(),
            scroll_handle: UniformListScrollHandle::new(),
        };
        file_tree.refresh_from_disk(cx);
        file_tree
    }

    pub fn activate_node(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let node = &self.nodes[&path];
        self.selected = Some(path.clone());
        if node.is_dir {
            if self.expanded_dirs.contains(&path) {
                self.expanded_dirs.remove(path.as_path());
            } else {
                self.expanded_dirs.insert(path);
            }
            self.rebuild_visible_rows();
        }
        cx.notify();
        // TODO: open file
    }

    pub fn refresh_from_disk(&mut self, cx: &mut Context<Self>) {
        let root = self.root.clone();
        let (tx, rx) = async_channel::unbounded::<Vec<(PathBuf, FileTreeNode)>>();

        cx.background_executor()
            .spawn(async move { Self::reload_nodes(root, tx).await })
            .detach();

        cx.spawn(async move |file_tree, cx| {
            while let Ok(batch) = rx.recv().await {
                _ = file_tree.update(cx, |file_tree, cx| {
                    file_tree.nodes.extend(batch);
                    file_tree.rebuild_visible_rows();
                    cx.notify();
                });
            }
            _ = file_tree.update(cx, |file_tree, cx| {
                file_tree.loading = false;
                cx.notify();
            });
        })
        .detach();
    }

    async fn reload_nodes(root: PathBuf, tx: async_channel::Sender<Vec<(PathBuf, FileTreeNode)>>) {
        let mut batch = Vec::new();
        for result in ignore::Walk::new(&root) {
            let Ok(entry) = result else {
                continue;
            };
            let path = entry.path().to_path_buf();
            let mut children = Vec::new();
            let is_dir = entry.file_type().map_or(false, |ft| ft.is_dir()) || path == root;
            let name = SharedString::from(entry.file_name().to_string_lossy());

            if is_dir {
                if let Ok(entries) = fs::read_dir(&path) {
                    let mut entries: Vec<_> = entries
                        .filter_map(|e| {
                            e.ok().map(|e| {
                                let p = e.path();
                                let is_dir = e.file_type().map_or(false, |ft| ft.is_dir());
                                (p, is_dir, e.file_name())
                            })
                        })
                        .collect();
                    entries.sort_by(|a, b| {
                        b.1.cmp(&a.1)
                            .then_with(|| a.2.to_ascii_lowercase().cmp(&b.2.to_ascii_lowercase()))
                    });
                    for (p, _, _) in entries {
                        children.push(p);
                    }
                }
            }

            let extension = if is_dir {
                None
            } else {
                path.extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| SharedString::from(ext.to_lowercase()))
            };
            batch.push((
                path.clone(),
                FileTreeNode {
                    path,
                    name,
                    extension,
                    children,
                    is_dir,
                },
            ));
            if batch.len() >= BATCH_SIZE {
                if tx.send(std::mem::take(&mut batch)).await.is_err() {
                    break;
                }
            }
        }
        if !batch.is_empty() {
            _ = tx.send(batch).await;
        }
    }

    fn rebuild_visible_rows(&mut self) {
        self.visible_nodes.clear();
        let mut stack = Vec::with_capacity(32);
        stack.push((&self.root, 0));
        while let Some((path, depth)) = stack.pop() {
            let Some(node) = self.nodes.get(path) else {
                continue;
            };
            let is_expanded = self.expanded_dirs.contains(path);
            self.visible_nodes.push(VisibleFileNode {
                index: self.visible_nodes.len(),
                path: path.clone(),
                depth,
                is_dir: node.is_dir,
                is_expanded,
                name: node.name.clone(),
                extension: node.extension.clone(),
            });
            if node.is_dir && is_expanded {
                for child in node.children.iter().rev() {
                    stack.push((&child, depth + 1));
                }
            }
        }
    }
}

impl Render for FileTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus)
            .key_context(FILE_TREE_CONTEXT)
            .size_full()
            .child(
                Progress::new("file-tree-loader")
                    .h_1()
                    .loading(self.loading)
                    .opacity(if self.loading { 1.0 } else { 0.0 }),
            )
            .child(
                uniform_list("file-tree", self.visible_nodes.len(), {
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        range
                            .into_iter()
                            .map(|i| {
                                let node = &this.visible_nodes[i];
                                let path = node.path.clone();
                                let icon = if !node.is_dir {
                                    Icon::new(IconName::File)
                                } else if node.is_expanded {
                                    Icon::new(IconName::FolderOpen)
                                } else {
                                    Icon::new(IconName::Folder)
                                };
                                ListItem::new(format!("tree-item-{}", node.index))
                                    .selected(this.selected.as_ref().is_some_and(|p| p == &path))
                                    .w_full()
                                    .p_0()
                                    .h_8()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.activate_node(path.clone(), cx);
                                    }))
                                    .child(
                                        h_flex()
                                            .child(h_flex().children(
                                                (0..node.depth).into_iter().map(|_| {
                                                    div()
                                                        .h_8()
                                                        .w_4()
                                                        .border_color(cx.theme().secondary)
                                                        .border_r_1()
                                                        .child("")
                                                }),
                                            ))
                                            .child(
                                                h_flex()
                                                    .pl_2()
                                                    .gap_2()
                                                    .child(icon)
                                                    .child(node.name.clone()),
                                            ),
                                    )
                            })
                            .collect()
                    })
                })
                .size_full()
                .track_scroll(&self.scroll_handle),
            )
            .scrollbar(&self.scroll_handle, ScrollbarAxis::Both)
    }
}
