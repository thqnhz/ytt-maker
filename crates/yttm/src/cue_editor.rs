use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;

use yttm_core::Cue;

use crate::app::YttmApp;

pub fn render_editor_area(
    cues: &[Cue],
    selected_cue: Option<u64>,
    cx: &Context<YttmApp>,
) -> impl IntoElement {
    let selected = selected_cue.and_then(|id| cues.iter().find(|cue| cue.id == id));
    div()
        .id("cue-editor-area")
        .flex()
        .flex_col()
        .flex_1()
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
                        .tooltip("Add cue")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.append_cue();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("add-cue-above")
                        .icon(IconName::ArrowUp)
                        .ghost()
                        .tooltip("Add cue above selection")
                        .disabled(selected_cue.is_none())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.add_cue_above_selected();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("add-cue-below")
                        .icon(IconName::ArrowDown)
                        .ghost()
                        .tooltip("Add cue below selection")
                        .disabled(selected_cue.is_none())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.add_cue_below_selected();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("delete-cue")
                        .icon(IconName::Minus)
                        .ghost()
                        .tooltip("Delete selected cue")
                        .disabled(selected_cue.is_none())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.delete_selected_cue();
                            cx.notify();
                        })),
                ),
        )
        .child(render_cue_table(cues, selected_cue, cx))
        .child(render_selected_details(selected, cx))
}

fn render_cue_table(
    cues: &[Cue],
    selected_cue: Option<u64>,
    cx: &Context<YttmApp>,
) -> impl IntoElement {
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
                .children(cues.iter().map(|cue| render_cue_row(cue, selected_cue, cx))),
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

fn render_cue_row(cue: &Cue, selected_cue: Option<u64>, cx: &Context<YttmApp>) -> impl IntoElement {
    let is_selected = selected_cue == Some(cue.id);
    let cue_id = cue.id;

    let mut row = div()
        .id(("cue-row", cue_id as usize))
        .flex()
        .items_center()
        .px_3()
        .h_8()
        .border_b_1()
        .border_color(cx.theme().border);

    if is_selected {
        row = row.bg(cx.theme().selection);
    }

    let mut subtitle = div().flex_1().min_w_0();
    if cue.text.is_empty() {
        subtitle = subtitle
            .text_color(cx.theme().muted_foreground)
            .child("(empty)")
    } else {
        subtitle = subtitle.child(cue.text.clone());
    }

    row.on_click(cx.listener(move |this, _, _, cx| {
        this.selected_cue = Some(cue_id);
        cx.notify();
    }))
    .child(div().w_24().child(format_ms(cue.start_ms)))
    .child(div().w_24().child(format_ms(cue.end_ms)))
    .child(subtitle)
    .child(div().w_16().child(format_opt_i32(cue.pos_x)))
    .child(div().w_16().child(format_opt_i32(cue.pos_y)))
}

fn render_selected_details(selected: Option<&Cue>, cx: &Context<YttmApp>) -> impl IntoElement {
    let mut details = div()
        .id("cue-detail")
        .flex()
        .flex_col()
        .justify_center()
        .gap_1()
        .px_3()
        .py_2()
        .border_t_1()
        .border_color(cx.theme().border);

    if let Some(cue) = selected {
        let mut subtitle = div();
        if cue.text.is_empty() {
            subtitle = subtitle
                .text_color(cx.theme().muted_foreground)
                .child("(empty)");
        } else {
            subtitle = subtitle.child(cue.text.clone())
        }

        details = details
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} - {} ({})",
                        format_ms(cue.start_ms),
                        format_ms(cue.end_ms),
                        format_duration_ms(cue.end_ms.saturating_sub(cue.start_ms))
                    ))
                    .child(div().flex_1().min_w_0())
                    .child(format!(
                        "Pos X: {} Pos Y: {}",
                        format_opt_i32(cue.pos_x),
                        format_opt_i32(cue.pos_y),
                    )),
            )
            .child(subtitle);
    } else {
        details = details.child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child("No cue selected"),
        );
    }

    details
}

fn format_duration_ms(ms: u64) -> String {
    let secs = ms / 1000;
    let millis = ms % 1000;
    format!("{secs}.{millis:03}s")
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
