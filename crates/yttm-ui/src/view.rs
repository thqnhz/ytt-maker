use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::base::{ResizeHandleRenderer, ResizeHandleState, resizable_panel};
use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::app::YttmApp;
use crate::cue_editor;
use crate::theme::*;
use crate::workspace::*;

fn no_hairline_resize_appearance() -> ResizeHandleRenderer {
    Rc::new(|handle, _window, cx| {
        let (length, opacity) = match handle.state() {
            ResizeHandleState::Idle => (px(0.), 0.0_f32),
            ResizeHandleState::Hovered => (px(30.), 0.35),
            ResizeHandleState::Pressed => (px(40.), 0.6),
            ResizeHandleState::Dragging => (px(52.), 0.9),
        };

        let mut line = div().flex_none().flex().bg(transparent_black());

        line = match handle.axis() {
            Axis::Horizontal => line.w(px(1.)).h_full().items_center(),
            Axis::Vertical => line.h(px(1.)).w_full().items_start().justify_center(),
        };

        if length > px(0.5) {
            let mut pill = div()
                .flex_none()
                .rounded(cx.theme().radius_full())
                .bg(cx.theme().muted_foreground)
                .opacity(opacity);
            pill = match handle.axis() {
                Axis::Horizontal => pill.w(px(4.)).h(length).ml(px(4.) * -0.5),
                Axis::Vertical => pill.h(px(4.)).w(length).mt(px(4.) * -0.5),
            };
            line = line.child(pill);
        }

        Some(line.into_any_element())
    })
}

impl Render for YttmApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace =
            match self.workspace.layout {
                WorkspaceLayout::Stacked => v_resizable("main-split-stacked")
                    .with_state(&self.workspace.split.stacked)
                    .with_handle_appearance(no_hairline_resize_appearance())
                    .child(
                        resizable_panel().size(px(400.)).child(
                            div()
                                .size_full()
                                .flex()
                                .flex_col()
                                .pb(px(4.))
                                .child(self.render_video_area(cx)),
                        ),
                    )
                    .child(resizable_panel().size_range(px(260.)..px(4000.)).child(
                        div().size_full().flex().flex_col().pt(px(4.)).child(
                            cue_editor::render_editor_area(
                                &self.document.cues,
                                self.selected_cue,
                                &self.input,
                                &self.cue_list_scroll,
                                cx,
                            ),
                        ),
                    )),
                WorkspaceLayout::SideBySide => {
                    h_resizable("main-split-side")
                        .with_state(&self.workspace.split.side_by_side)
                        .with_handle_appearance(no_hairline_resize_appearance())
                        .child(
                            resizable_panel().size(px(480.)).child(
                                div()
                                    .size_full()
                                    .flex()
                                    .pr(px(4.))
                                    .child(self.render_video_area(cx)),
                            ),
                        )
                        .child(resizable_panel().size_range(px(520.)..px(4000.)).child(
                            div().size_full().flex().pl(px(4.)).child(
                                cue_editor::render_editor_area(
                                    &self.document.cues,
                                    self.selected_cue,
                                    &self.input,
                                    &self.cue_list_scroll,
                                    cx,
                                ),
                            ),
                        ))
                }
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
        let is_dark = cx.theme().mode.is_dark();
        let theme_tooltip = theme_toggle_tooltip(is_dark);
        let theme_icon = theme_toggle_icon(is_dark);
        let layout_tooltip = Workspace::layout_tooltip(self.workspace.layout);
        let layout_icon = Workspace::layout_icon(self.workspace.layout);

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
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        Button::new("open-file")
                            .icon(IconName::FolderOpen)
                            .ghost()
                            .tooltip("Open")
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        Button::new("save-file")
                            .icon(IconName::Save)
                            .ghost()
                            .tooltip("Save")
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        Button::new("toggle-theme")
                            .icon(theme_icon)
                            .ghost()
                            .tooltip(theme_tooltip)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.theme.toggle_theme(cx);
                            })),
                    )
                    .child(
                        Button::new("toggle-layout")
                            .icon(layout_icon)
                            .ghost()
                            .tooltip(layout_tooltip)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.workspace.switch_layout();
                                cx.notify();
                            })),
                    ),
            )
    }

    fn render_video_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let subtitle_text = self.selected_cue.and_then(|id| {
            self.document
                .cues
                .iter()
                .find(|cue| cue.id == id)
                .map(|cue| cue.text.clone())
                .filter(|text| !text.trim().is_empty())
        });

        let mut subtitle_bar = div()
            .w_full()
            .min_h(px(40.))
            .flex()
            .items_center()
            .justify_center()
            .pb_3()
            .px_4();
        if let Some(text) = subtitle_text {
            subtitle_bar = subtitle_bar.child(div().text_color(gpui::white()).child(text));
        }

        let screen = div()
            .id("video-screen")
            .flex()
            .flex_col()
            .bg(gpui::black())
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .items_center()
                    .justify_center()
                    .child(div().text_color(gpui::white()).child("No video loaded")),
            )
            .child(subtitle_bar);

        let content = match self.workspace.layout {
            WorkspaceLayout::SideBySide => div()
                .id("video-content")
                .size_full()
                .flex()
                .flex_row()
                .items_start()
                .min_w_0()
                .min_h_0()
                .child(
                    screen
                        .h_full()
                        .max_w_full()
                        .min_w_0()
                        .aspect_ratio(16.0 / 9.0),
                )
                .child(div().id("video-side-pane").flex_1().min_w_0().h_full()),
            WorkspaceLayout::Stacked => div()
                .id("video-content")
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .min_w_0()
                .min_h_0()
                .child(
                    screen
                        .w_full()
                        .max_h_full()
                        .min_h_0()
                        .aspect_ratio(16.0 / 9.0),
                )
                .child(div().id("video-bottom-pane").flex_1().min_h_0().w_full()),
        };

        div()
            .id("video-area")
            .flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted)
            .child(content)
    }
}
