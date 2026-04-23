use gpui::{actions, App, Context, Window};
use paths::config_dir;
use std::{borrow::Borrow, fs};
use workspace::{notifications::NotifyResultExt, Workspace};

actions!(baptized, [OpenFolder]);

pub fn init(cx: &mut App) {
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
