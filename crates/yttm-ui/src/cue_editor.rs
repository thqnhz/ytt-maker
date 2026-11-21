use gpui_kit::assets::IconName;
use gpui_kit::base::input::InputBaseState;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::*;
use gpui_kit::component::input::{Enter, Escape, Input, InputMode, Textarea};
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

use yttm_core::Cue;
use yttm_input::YttmInput;

use crate::app::YttmApp;
use crate::format::*;

pub fn render_editor_area(
    cues: &[Cue],
    selected_cue: Option<u64>,
    input: &YttmInput,
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
        .child(render_cue_editor_toolbar(selected_cue, cx))
        .child(render_cue_table(cues, selected_cue, cx))
        .child(render_selected_details(selected, input, cx))
}

fn render_cue_editor_toolbar(selected_cue: Option<u64>, cx: &Context<YttmApp>) -> impl IntoElement {
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
                .on_click(cx.listener(|this, _, window, cx| {
                    this.append_cue();
                    this.sync_selected_cue_inputs(window, cx);
                    this.focus_subtitle(window, cx);
                    cx.notify();
                })),
        )
        .child(
            Button::new("add-cue-above")
                .icon(IconName::ArrowUp)
                .ghost()
                .tooltip("Add cue above selection")
                .disabled(selected_cue.is_none())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.add_cue_above_selected();
                    this.sync_selected_cue_inputs(window, cx);
                    this.focus_subtitle(window, cx);
                    cx.notify();
                })),
        )
        .child(
            Button::new("add-cue-below")
                .icon(IconName::ArrowDown)
                .ghost()
                .tooltip("Add cue below selection")
                .disabled(selected_cue.is_none())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.add_cue_below_selected();
                    this.sync_selected_cue_inputs(window, cx);
                    this.focus_subtitle(window, cx);
                    cx.notify();
                })),
        )
        .child(
            Button::new("duplicate-cue")
                .icon(IconName::Copy)
                .ghost()
                .tooltip("Duplicate selected cue")
                .disabled(selected_cue.is_none())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.duplicate_selected_cue();
                    this.sync_selected_cue_inputs(window, cx);
                    this.focus_subtitle(window, cx);
                    cx.notify();
                })),
        )
        .child(
            Button::new("delete-cue")
                .icon(IconName::Minus)
                .ghost()
                .tooltip("Delete selected cue")
                .disabled(selected_cue.is_none())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.delete_selected_cue();
                    this.sync_selected_cue_inputs(window, cx);
                    if this.selected_cue.is_some() {
                        this.focus_subtitle(window, cx);
                    }
                    cx.notify();
                })),
        )
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
        .child(render_cue_rows(cues, selected_cue, cx))
}

fn render_cue_header(cx: &Context<YttmApp>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .pr_3()
        .h_8()
        .border_b_1()
        .border_color(cx.theme().border)
        .text_color(cx.theme().muted_foreground)
        .child(div().w_8())
        .child(div().w_24().child("Start"))
        .child(div().w_24().child("End"))
        .child(div().flex_1().min_w_0().child("Subtitle"))
        .child(div().w_16().child("Pos X"))
        .child(div().w_16().child("Pos Y"))
}

fn render_cue_rows(
    cues: &[Cue],
    selected_cue: Option<u64>,
    cx: &Context<YttmApp>,
) -> impl IntoElement {
    div()
        .id("cue-rows")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .when(cues.is_empty(), |el| {
            el.child(
                div()
                    .id("cue-empty")
                    .w_full()
                    .py_4()
                    .flex()
                    .justify_center()
                    .text_color(cx.theme().muted_foreground)
                    .child("No cues yet - press + to add one."),
            )
        })
        .children(cues.iter().map(|cue| render_cue_row(cue, selected_cue, cx)))
}

fn render_cue_row(cue: &Cue, selected_cue: Option<u64>, cx: &Context<YttmApp>) -> impl IntoElement {
    let is_selected = selected_cue == Some(cue.id);
    let cue_id = cue.id;
    let invalid = cue.end_ms < cue.start_ms;

    let mut row = div()
        .id(("cue-row", cue_id as usize))
        .flex()
        .items_center()
        .pr_3()
        .min_h_8()
        .border_b_1()
        .border_color(cx.theme().border);

    if is_selected {
        row = row.bg(cx.theme().selection);
    }

    let mut status = div().w_8().flex().justify_center();
    if invalid {
        status = status
            .text_color(cx.theme().danger)
            .child(IconName::CircleAlert);
    }

    let mut subtitle = div().flex_1().min_w_0();
    if cue.text.is_empty() {
        subtitle = subtitle
            .text_color(cx.theme().muted_foreground)
            .child("(empty)")
    } else {
        subtitle = subtitle.flex().flex_col().child(cue.text.clone());
    }

    let pos_x = if let Some(v) = cue.pos_x {
        div().w_16().child(v.to_string())
    } else {
        div()
            .w_16()
            .text_color(cx.theme().muted_foreground)
            .child("-")
    };
    let pos_y = if let Some(v) = cue.pos_y {
        div().w_16().child(v.to_string())
    } else {
        div()
            .w_16()
            .text_color(cx.theme().muted_foreground)
            .child("-")
    };

    row.on_click(cx.listener(move |this, _, window, cx| {
        this.selected_cue = Some(cue_id);
        this.sync_selected_cue_inputs(window, cx);
        this.focus_subtitle(window, cx);
        cx.notify();
    }))
    .child(status)
    .child(div().w_24().child(format_ms(cue.start_ms)))
    .child(div().w_24().child(format_ms(cue.end_ms)))
    .child(subtitle)
    .child(pos_x)
    .child(pos_y)
}

fn render_selected_details(
    selected: Option<&Cue>,
    input: &YttmInput,
    cx: &Context<YttmApp>,
) -> impl IntoElement {
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
        details = details
            .child(
                div()
                    .flex()
                    .flex_row()
                    .child(render_time_fields(cue, input, cx))
                    .child(div().flex_1())
                    .child(render_pos_fields(input, cx)),
            )
            .child(render_subtitle_field(input, cx));
    } else {
        details = details.child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child("No cue selected"),
        );
    }
    details
}

fn render_time_fields(cue: &Cue, input: &YttmInput, cx: &Context<YttmApp>) -> impl IntoElement {
    let invalid = cue.end_ms < cue.start_ms;
    let duration = if invalid {
        div()
            .text_color(cx.theme().danger)
            .child("(invalid: end time before start time)")
    } else {
        div().text_color(cx.theme().muted_foreground).child(format!(
            "({})",
            format_duration_ms(cue.end_ms.saturating_sub(cue.start_ms))
        ))
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(meta_label("Start", cx))
        .child(time_input(&input.start, cx))
        .child(meta_label("End", cx))
        .child(time_input(&input.end, cx))
        .child(duration)
}

fn render_pos_fields(input: &YttmInput, cx: &Context<YttmApp>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(meta_label("Pos X", cx))
        .child(
            div()
                .w_12()
                .on_action(cx.listener(|this, _: &Enter, window, cx| {
                    this.commit_selected_cue_inputs(window, cx);
                    window.blur(cx);
                }))
                .on_action(cx.listener(|this, _: &Escape, window, cx| {
                    this.sync_selected_cue_inputs(window, cx);
                    window.blur(cx);
                }))
                .child(Input::new(&input.pos_x)),
        )
        .child(meta_label("Pos Y", cx))
        .child(
            div()
                .w_12()
                .on_action(cx.listener(|this, _: &Enter, window, cx| {
                    this.commit_selected_cue_inputs(window, cx);
                    window.blur(cx);
                }))
                .on_action(cx.listener(|this, _: &Escape, window, cx| {
                    this.sync_selected_cue_inputs(window, cx);
                    window.blur(cx);
                }))
                .child(Input::new(&input.pos_y)),
        )
}

fn meta_label(text: &'static str, cx: &Context<YttmApp>) -> impl IntoElement {
    div()
        .w_12()
        .text_color(cx.theme().muted_foreground)
        .child(text)
}

fn time_input(
    input: &Entity<InputBaseState<InputMode>>,
    cx: &Context<YttmApp>,
) -> impl IntoElement {
    div()
        .w_24()
        .on_action(cx.listener(|this, _: &Enter, window, cx| {
            this.commit_selected_cue_inputs(window, cx);
            window.blur(cx);
        }))
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            this.sync_selected_cue_inputs(window, cx);
            window.blur(cx);
        }))
        .child(Input::new(input))
}

fn render_subtitle_field(input: &YttmInput, cx: &Context<YttmApp>) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_start()
        .gap_1()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(Textarea::new(&input.subtitle)),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("revert-cue")
                        .icon(IconName::Close)
                        .ghost()
                        .text_color(cx.theme().danger)
                        .tooltip("Revert changes")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.sync_selected_cue_inputs(window, cx);
                            window.blur(cx);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("commit-cue")
                        .icon(IconName::Check)
                        .ghost()
                        .text_color(cx.theme().success)
                        .tooltip("Commit changes")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.commit_selected_cue_inputs(window, cx);
                            window.blur(cx);
                        })),
                ),
        )
}
