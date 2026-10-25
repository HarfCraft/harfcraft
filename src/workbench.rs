use crate::{
    APP_DESCRIPTION, APP_NAME,
    actions::{OpenWorkspace, WORKBENCH_CONTEXT},
    dragged_workspace::DraggedWorkspace,
    prelude::*,
    workspace::Workspace,
};
use gpui_kit::{
    PathPromptOptions,
    base::StyledExt,
    component::{
        Icon, TitleBar,
        label::Label,
        menu::DropdownMenu,
        separator::Separator,
        tab::{Tab, TabBar},
        text::Text,
    },
};
use std::path::PathBuf;

pub struct Workbench {
    focus: FocusHandle,
    workspaces: Vec<Entity<Workspace>>,
    active: Option<usize>,
}

impl Focusable for Workbench {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Workbench {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Self {
            focus,
            workspaces: Vec::new(),
            active: None,
        }
    }

    pub fn activate_workspace(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.workspaces.len() && self.active != Some(index) {
            self.active = Some(index);
            cx.notify();
        }
    }
    pub fn activate_welcome_view(&mut self, cx: &mut Context<Self>) {
        if self.active.is_some() {
            self.active = None;
            cx.notify();
        }
    }
    pub fn close_workspace(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.workspaces.len() {
            return;
        }
        self.workspaces.remove(index);
        self.active = match self.active {
            Some(i) if i > index => Some(i - 1),
            Some(i) if i == index => {
                (!self.workspaces.is_empty()).then(|| i.min(self.workspaces.len() - 1))
            }
            other => other,
        };
        cx.notify();
    }
    pub fn load_workspace(&mut self, root: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let root = root.canonicalize().unwrap_or(root);
        if let Some(index) = self.workspaces.iter().position(|w| w.read(cx).root == root) {
            self.activate_workspace(index, cx);
            return;
        }
        let workspace = cx.new(|cx| Workspace::new(root, window, cx));
        self.workspaces.push(workspace);
        self.activate_workspace(self.workspaces.len() - 1, cx);
    }
    pub fn move_workspace(&mut self, from: usize, to: usize, cx: &mut Context<Self>) {
        if from == to || from >= self.workspaces.len() || to >= self.workspaces.len() {
            return;
        }
        let workspace = self.workspaces.remove(from);
        self.workspaces.insert(to, workspace);
        self.active = Some(to);
        cx.notify();
    }

    pub fn open_workspace(
        &mut self,
        _: &OpenWorkspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open Workspace".into()),
        });
        _ = cx
            .spawn_in(window, async move |workbench, cx| {
                let root = prompt.await.ok()?.ok()??.first()?.clone();
                _ = workbench.update_in(cx, |workbench, window, cx| {
                    workbench.load_workspace(root, window, cx);
                });
                Some(())
            })
            .detach();
    }
}

impl Render for Workbench {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(WORKBENCH_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::open_workspace))
            .size_full()
            .child(
                TitleBar::new().child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("app-menu")
                                .ghost()
                                .small()
                                .icon(IconName::Menu)
                                .dropdown_menu(|menu, _, _| {
                                    menu.menu("Open workspace", Box::new(OpenWorkspace))
                                }),
                        )
                        .child(Text::from(APP_NAME))
                        .child(
                            TabBar::new("workspaces-tab-bar")
                                .segmented()
                                .on_click(cx.listener(|this, &index: &usize, _, cx| {
                                    if index >= this.workspaces.len() {
                                        this.activate_welcome_view(cx);
                                    } else {
                                        this.activate_workspace(index, cx);
                                    }
                                }))
                                .selected_index(self.active.unwrap_or(self.workspaces.len()))
                                .children(self.workspaces.iter().enumerate().map(
                                    |(index, workspace)| {
                                        let workspace = workspace.read(cx);
                                        Tab::new()
                                            .label(workspace.name.clone())
                                            .on_drag(
                                                DraggedWorkspace::new(
                                                    index,
                                                    workspace.name.clone(),
                                                ),
                                                |drag, _, _, cx| cx.new(|_| drag.clone()),
                                            )
                                            .drag_over::<DraggedWorkspace>(
                                                move |style, drag, _, cx| {
                                                    if drag.index > index {
                                                        style
                                                            .rounded_none()
                                                            .border_l_2()
                                                            .border_color(cx.theme().drag_border)
                                                    } else if drag.index < index {
                                                        style
                                                            .rounded_none()
                                                            .border_r_2()
                                                            .border_color(cx.theme().drag_border)
                                                    } else {
                                                        style.bg(cx.theme().drop_target)
                                                    }
                                                },
                                            )
                                            .on_drop(cx.listener::<DraggedWorkspace>(
                                                move |this, drag, _, cx| {
                                                    this.move_workspace(drag.index, index, cx)
                                                },
                                            ))
                                            .pl_2()
                                            .prefix(Icon::new(IconName::BookOpen).small())
                                            .suffix(
                                                Button::new(format!("close-workspace-{index}"))
                                                    .small()
                                                    .ghost()
                                                    .icon(IconName::Close)
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.close_workspace(index, cx);
                                                            cx.stop_propagation();
                                                        },
                                                    )),
                                            )
                                    },
                                ))
                                .child(Tab::new().icon(IconName::Plus)),
                        ),
                ),
            )
            .when_some(
                self.active.and_then(|i| self.workspaces.get(i).cloned()),
                |this, workspace| this.child(workspace),
            )
            .when(
                self.active.is_none_or(|i| i >= self.workspaces.len()),
                |this| {
                    this.child(
                        v_flex()
                            .justify_center()
                            .items_stretch()
                            .w_128()
                            .h_full()
                            .mx_auto()
                            .gap_4()
                            .text_center()
                            .child(Label::new(APP_NAME).text_3xl().font_bold())
                            .child(
                                Label::new(APP_DESCRIPTION)
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                Button::new("open-workspace")
                                    .ghost()
                                    .label("Open Workspace")
                                    .icon(IconName::FolderOpen)
                                    .tooltip_with_action(
                                        "Open workspace",
                                        &OpenWorkspace,
                                        Some(WORKBENCH_CONTEXT),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_workspace(&OpenWorkspace, window, cx);
                                    })),
                            )
                            .child(
                                h_flex()
                                    .w_128()
                                    .gap_2()
                                    .child(Separator::horizontal().flex_1())
                                    .child(
                                        Label::new("RECENT PROJECTS")
                                            .text_color(cx.theme().muted_foreground)
                                            .text_xs(),
                                    )
                                    .child(Separator::horizontal().flex_1()),
                            ),
                    )
                },
            )
    }
}
