use gpui_kit::{
    WindowOptions,
    base::Root,
    component::{Theme, ThemeRegistry, TitleBar},
};
use harfcraft::{APP_ID, APP_NAME, actions, logging, prelude::*, workbench::Workbench};

fn main() {
    logging::init();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            actions::init(cx);

            cx.set_app_identity(APP_ID, APP_NAME);
            cx.activate(true);

            let theme = ThemeRegistry::global(cx).default_dark_theme().clone();
            // Keep this so that dock area separator does not appear white
            Theme::change(theme.mode, None, cx);
            Theme::global_mut(cx).apply_config(&theme);

            let window_options = WindowOptions {
                app_id: Some(APP_ID.into()),
                window_decorations: Some(gpui_kit::WindowDecorations::Client),
                ..TitleBar::window_options()
            };
            cx.spawn(async move |cx| {
                cx.open_window(window_options, |window, cx| {
                    let workbench = cx.new(|cx| Workbench::new(window, cx));
                    cx.new(|cx| Root::new(workbench, window, cx))
                })
                .expect("Failed to open window");
            })
            .detach();
        })
}
