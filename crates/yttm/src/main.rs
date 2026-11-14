use gpui_kit::assets::IconName;
use gpui_kit::base::resizable_panel;
use gpui_kit::component::button::*;
use gpui_kit::component::{ActiveTheme as _, h_resizable, v_resizable};
use gpui_kit::*;

mod app;
mod cue_editor;

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
                .child(
                    resizable_panel()
                        .child(cue_editor::render_editor_area(&self.document.cues, cx)),
                ),
            WorkspaceLayout::SideBySide => h_resizable("main-split-side")
                .with_state(&self.side_split)
                .child(
                    resizable_panel()
                        .size(px(480.))
                        .child(self.render_video_area(cx)),
                )
                .child(
                    resizable_panel()
                        .child(cue_editor::render_editor_area(&self.document.cues, cx)),
                ),
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
        let (layout_label, layout_icon) = match self.workspace_layout {
            WorkspaceLayout::Stacked => ("Switch to Side by side", IconName::PanelBottom),
            WorkspaceLayout::SideBySide => ("Switch to Stacked", IconName::PanelRight),
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
                        Button::new("toggle-layout")
                            .icon(layout_icon)
                            .ghost()
                            .tooltip(layout_label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.workspace_layout = match this.workspace_layout {
                                    WorkspaceLayout::Stacked => WorkspaceLayout::SideBySide,
                                    WorkspaceLayout::SideBySide => WorkspaceLayout::Stacked,
                                };
                                cx.notify();
                            })),
                    ),
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
}

fn main() {
    YttmApp::run();
}
