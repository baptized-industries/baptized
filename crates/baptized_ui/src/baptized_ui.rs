use editor::Editor;
use gpui::{
    App, Context, Entity, EventEmitter, Focusable, ParentElement, Render, Styled, Window, actions,
};
use paths::config_dir;
use std::{borrow::Borrow, fs};
use ui::prelude::*;
use workspace::{
    Workspace,
    item::{Item, ItemEvent},
    notifications::NotifyResultExt,
};

use pyo3::prelude::*;
use pyo3_ffi::c_str;

actions!(baptized, [OpenFolder]);
actions!(baptized_page, [ToggleFocus]);

pub fn init(cx: &mut App) {
    cx.observe_new(register).detach();
}

fn register(workspace: &mut Workspace, _window: Option<&mut Window>, _: &mut Context<Workspace>) {
    workspace.register_action(open_folder);
    workspace.register_action(toggle_focus);
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

fn toggle_focus(
    workspace: &mut Workspace,
    _: &ToggleFocus,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let existing = workspace
        .active_pane()
        .read(cx)
        .items()
        .find_map(|item| item.downcast::<BaptizedPage>());

    if let Some(existing) = existing {
        workspace.activate_item(&existing, true, true, window, cx);
    } else {
        let baptized_page = BaptizedPage::new(window, cx);
        workspace.add_item_to_active_pane(Box::new(baptized_page), None, true, window, cx)
    }
}

pub struct BaptizedPage {
    query_editor: Entity<Editor>,
    version: String,
}

fn helper() -> PyResult<String> {
    Python::attach(|py| {
        let module = PyModule::from_code(
            py,
            c_str!(
                r#"
import signal

import pandas as pd

# numpy messes up keyboard interrupts, this restores defaults
signal.signal(signal.SIGINT, signal.SIG_DFL)

def wrapper():
    data = {"col1": ["val1"], "col2": ["val2"]}
    df = pd.DataFrame.from_dict(data)
    return str(df.iloc[0].col1)
"#
            ),
            c_str!("module.py"),
            c_str!("module"),
        )?;

        let version: String = module.getattr("wrapper")?.call0()?.extract()?;
        Ok(version)
    })
}

impl BaptizedPage {
    pub fn new(window: &mut Window, cx: &mut Context<Workspace>) -> Entity<Self> {
        cx.new(|cx| {
            let query_editor = cx.new(|cx| {
                let mut input = Editor::single_line(window, cx);
                input.set_placeholder_text("Search extensions...", window, cx);
                input
            });

            let version: String = helper().unwrap();

            let this = Self {
                query_editor,
                version,
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
                            .child(Headline::new(&self.version).size(HeadlineSize::XLarge)),
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
