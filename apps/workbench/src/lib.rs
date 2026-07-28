mod actions;
mod models;
mod theme;
mod workbench;

use actions::*;
use gpui::{
    prelude::*, px, size, App, Application, Bounds, KeyBinding, WindowBounds, WindowOptions,
};
use polyglid_client::{client::LocalClient, controllers::DesktopControllers};
use workbench::Workbench;

pub fn launch() {
    let client = LocalClient::open_default()
        .unwrap_or_else(|error| panic!("failed to open the PolyGlid workspace: {error}"));
    let controllers = DesktopControllers::new(client);
    let snapshot = controllers
        .application
        .bootstrap()
        .unwrap_or_else(|error| panic!("failed to load the PolyGlid workspace: {error}"));

    Application::new().run(move |cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("ctrl-p", OpenCommandPalette, None),
            KeyBinding::new("ctrl-b", ToggleSidebar, None),
            KeyBinding::new("ctrl-j", ToggleBottomPanel, None),
            KeyBinding::new("ctrl-1", OpenProjects, None),
            KeyBinding::new("ctrl-2", OpenScanner, None),
            KeyBinding::new("ctrl-3", OpenExecutions, None),
            KeyBinding::new("ctrl-4", OpenReports, None),
            KeyBinding::new("ctrl-5", OpenPlugins, None),
            KeyBinding::new("ctrl-w", CloseActiveTab, None),
            KeyBinding::new("ctrl-tab", NextTab, None),
            KeyBinding::new("ctrl-r", RefreshWorkspace, None),
            KeyBinding::new("escape", CloseOverlay, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.), px(600.))),
                app_id: Some("dev.polyglid.workbench".to_owned()),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Workbench::new(controllers, snapshot, cx)),
        )
        .expect("failed to open the PolyGlid workbench");
        cx.activate(true);
    });
}
