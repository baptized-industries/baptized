use editor::Editor;
use gpui::{
    App, Context, Entity, EventEmitter, Focusable, ParentElement, Render, Styled, WeakEntity,
    Window, actions,
};
use paths::config_dir;
use std::{borrow::Borrow, fs};
use ui::prelude::*;
use workspace::{
    Workspace,
    item::{Item, ItemEvent},
    notifications::NotifyResultExt,
};

actions!(baptized, [OpenFolder, ToggleFocus]);

pub fn init(cx: &mut App) {
    cx.observe_new(move |workspace: &mut Workspace, _window, _| {
        workspace.register_action(move |workspace, _: &ToggleFocus, window, cx| {
            let existing = workspace
                .active_pane()
                .read(cx)
                .items()
                .find_map(|item| item.downcast::<BaptizedPage>());

            if let Some(existing) = existing {
                workspace.activate_item(&existing, true, true, window, cx);
            } else {
                let baptized_page = BaptizedPage::new(workspace, window, cx);
                workspace.add_item_to_active_pane(Box::new(baptized_page), None, true, window, cx)
            }
        });
    })
    .detach();

    cx.observe_new(register).detach();
}

fn register(workspace: &mut Workspace, _window: Option<&mut Window>, _: &mut Context<Workspace>) {
    workspace.register_action(open_folder);
}

fn open_folder(
    workspace: &mut Workspace,
    _: &OpenFolder,
    _: &mut Window,
    cx: &mut Context<Workspace>,
) {
    fs::create_dir_all(config_dir().join("baptized")).notify_err(workspace, cx);
    cx.open_with_system(config_dir().join("baptized").borrow());
}

pub struct BaptizedPage {
    workspace: WeakEntity<Workspace>,
    query_editor: Entity<Editor>,
}

impl BaptizedPage {
    pub fn new(
        workspace: &Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let query_editor = cx.new(|cx| {
                let mut input = Editor::single_line(window, cx);
                input.set_placeholder_text("Search extensions...", window, cx);
                input
            });

            let this = Self {
                workspace: workspace.weak_handle(),
                query_editor,
            };
            this
        })
    }
}

impl Render for BaptizedPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .gap_4()
                    .pt_4()
                    .px_4()
                    .bg(cx.theme().colors().editor_background)
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .justify_between()
                            .child(Headline::new("Baptized").size(HeadlineSize::XLarge)),
                    ),
            )
    }
}

impl EventEmitter<ItemEvent> for BaptizedPage {}

impl Focusable for BaptizedPage {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.query_editor.read(cx).focus_handle(cx)
    }
}

impl Item for BaptizedPage {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Baptized".into()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Baptized Page Opened")
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn to_item_events(event: &Self::Event, mut f: impl FnMut(workspace::item::ItemEvent)) {
        f(*event)
    }
}
