use std::rc::Rc;

use gpui_kit::base::{ResizeHandleRenderer, ResizeHandleState, resizable_panel};
use gpui_kit::component::*;
use gpui_kit::*;

use crate::app::YttmApp;
use crate::button::{open_button, save_button, toggle_layout_button, toggle_theme_button};
use crate::cue_editor;
use crate::workspace::*;
use gpui_kit::component::select::Select;

gpui_kit::actions!(yttm, [Undo, Redo]);

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
                                self.can_undo(),
                                self.can_redo(),
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
                                    self.can_undo(),
                                    self.can_redo(),
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
            .on_action(cx.listener(|this, _: &Undo, window, cx| this.undo(window, cx)))
            .on_action(cx.listener(|this, _: &Redo, window, cx| this.redo(window, cx)))
    }
}

impl YttmApp {
    fn render_top_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child(open_button(cx))
                    .child(save_button())
                    .child(toggle_theme_button(cx))
                    .child(toggle_layout_button(&self.workspace.layout, cx)),
            )
    }

    fn render_video_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let subtitle_text = self
            .selected_cue()
            .map(|cue| cue.text.clone())
            .filter(|text| !text.trim().is_empty());

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

        let center_label = self
            .video
            .file_name()
            .unwrap_or_else(|| "No video loaded".to_owned());

        let mut controls = div()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .py_1()
            .text_color(gpui::white());
        if self.video.path.is_some() {
            let play_label = if self.video.playing { "Pause" } else { "Play" };
            controls = controls
                .child(
                    div()
                        .id("video-play-toggle")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded(cx.theme().radius)
                        .bg(gpui::rgba(0xffffff22))
                        .hover(|s| s.bg(gpui::rgba(0xffffff33)))
                        .child(play_label)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_video_playback(cx);
                        })),
                )
                .child(
                    div().w(px(64.)).child(
                        Select::new(&self.preview_quality_select)
                            .id("video-preview-quality")
                            .menu_width(px(64.)),
                    ),
                );
        }

        let video_content = if let Some(frame) = &self.video_frame {
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .items_center()
                .justify_center()
                .child(
                    img(ImageSource::Render(frame.clone()))
                        .size_full()
                        .object_fit(ObjectFit::Contain),
                )
        } else {
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .items_center()
                .justify_center()
                .child(div().text_color(gpui::white()).child(center_label))
        };

        let screen = div()
            .id("video-screen")
            .flex()
            .flex_col()
            .bg(gpui::black())
            .child(video_content)
            .child(controls)
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
