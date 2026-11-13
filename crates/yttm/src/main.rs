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
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_top_bar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_video_area(cx))
                    .child(self.render_editor_area(cx)),
            )
    }
}

impl YttmApp {
    fn render_top_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .px_4()
            .h_10()
            .border_b_1()
            .border_color(cx.theme().border)
            .child("YTT Maker")
    }

    fn render_video_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("video-area")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .items_center()
            .justify_center()
            .m_3()
            .gap_1()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted)
            .child("Video preview")
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child("Player + subtitle overlay"),
            )
    }

    fn render_editor_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("cue-editor-area")
            .flex()
            .flex_col()
            .h_64()
            .min_h_0()
            .mx_3()
            .mb_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted)
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_3()
                    .h_8()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child("Cue editor"),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .items_center()
                    .justify_center()
                    .text_color(cx.theme().muted_foreground)
                    .child("Spreadsheet placeholder"),
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
