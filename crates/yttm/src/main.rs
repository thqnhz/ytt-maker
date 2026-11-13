use gpui_kit::base::resizable_panel;
use gpui_kit::component::button::*;
use gpui_kit::component::{ActiveTheme as _, h_resizable, v_resizable};
use gpui_kit::*;

use yttm_core::Cue;

mod app;

use crate::app::{WorkspaceLayout, YttmApp};

impl Render for YttmApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = match self.workspace_layout {
            WorkspaceLayout::Stacked => v_resizable("main-split-stacked")
                .with_state(&self.stacked_split)
                .child(
                    resizable_panel()
                        .size(px(400.))
                        .child(self.render_video_area(cx)),
                )
                .child(resizable_panel().child(self.render_editor_area(cx))),
            WorkspaceLayout::SideBySide => h_resizable("main-split-side")
                .with_state(&self.side_split)
                .child(
                    resizable_panel()
                        .size(px(480.))
                        .child(self.render_video_area(cx)),
                )
                .child(resizable_panel().child(self.render_editor_area(cx))),
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_top_bar(cx))
            .child(
                div()
                    .id("workspace")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .p_3()
                    .child(workspace),
            )
    }
}

impl YttmApp {
    fn render_top_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let toggle_label = match self.workspace_layout {
            WorkspaceLayout::Stacked => "Side by side",
            WorkspaceLayout::SideBySide => "Stacked",
        };
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .h_10()
            .border_b_1()
            .border_color(cx.theme().border)
            .child("YTT Maker")
            .child(
                Button::new("toggle-layout")
                    .label(toggle_label)
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.workspace_layout = match this.workspace_layout {
                            WorkspaceLayout::Stacked => WorkspaceLayout::SideBySide,
                            WorkspaceLayout::SideBySide => WorkspaceLayout::Stacked,
                        };
                        cx.notify();
                    })),
            )
    }

    fn render_video_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("video-area")
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .items_center()
            .justify_center()
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
            .size_full()
            .min_h_0()
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
            .child(self.render_cue_table(cx))
    }

    fn render_cue_table(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(render_cue_header(cx))
            .child(
                div()
                    .id("cue-rows")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(self.document.cues.iter().map(|cue| render_cue_row(cue, cx))),
            )
    }
}

fn render_cue_header(cx: &Context<YttmApp>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .px_3()
        .h_8()
        .border_b_1()
        .border_color(cx.theme().border)
        .text_color(cx.theme().muted_foreground)
        .child(div().w_24().child("Start"))
        .child(div().w_24().child("End"))
        .child(div().flex_1().min_w_0().child("Subtitle"))
        .child(div().w_16().child("Pos X"))
        .child(div().w_16().child("Pos Y"))
}

fn render_cue_row(cue: &Cue, cx: &Context<YttmApp>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .px_3()
        .h_8()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(div().w_24().child(format_ms(cue.start_ms)))
        .child(div().w_24().child(format_ms(cue.end_ms)))
        .child(div().flex_1().min_w_0().child(cue.text.clone()))
        .child(div().w_16().child(format_opt_i32(cue.pos_x)))
        .child(div().w_16().child(format_opt_i32(cue.pos_y)))
}

fn format_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = total_secs / 3600;

    if hours > 0 {
        format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
    } else {
        format!("{mins:02}:{secs:02}.{millis:03}")
    }
}

fn format_opt_i32(value: Option<i32>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
}

fn main() {
    YttmApp::run();
}
