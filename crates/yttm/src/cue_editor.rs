use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::*;
use gpui_kit::*;

use yttm_core::Cue;

use crate::app::YttmApp;

pub fn render_editor_area(cues: &[Cue], cx: &Context<YttmApp>) -> impl IntoElement {
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
                .id("cue-editor-toolbar")
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .h_8()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    Button::new("add-cue")
                        .icon(IconName::Plus)
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.add_cue();
                            cx.notify();
                        })),
                ),
        )
        .child(render_cue_table(cues, cx))
}

fn render_cue_table(cues: &[Cue], cx: &Context<YttmApp>) -> impl IntoElement {
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
                .children(cues.iter().map(|cue| render_cue_row(cue, cx))),
        )
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
