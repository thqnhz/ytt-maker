use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

struct YttmApp;

impl YttmApp {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for YttmApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child("YTT Maker")
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child("Placeholder"),
            )
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| YttmApp::new(window, cx))
            })
            .expect("failed to open window");
        });
}
