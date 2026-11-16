use std::borrow::Cow;

use gpui_kit::base::ResizableState;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::*;
use gpui_kit::*;
use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::base::{ResizeHandleRenderer, ResizeHandleState, resizable_panel};
use gpui_kit::component::button::*;
use gpui_kit::component::{h_resizable, v_resizable};

use crate::cue_editor;
use crate::theme::YttmTheme;
use yttm_core::{Cue, Document, WorkspaceLayout};
use yttm_input::YttmInput;

pub struct YttmApp {
    pub document: Document,
    pub selected_cue: Option<u64>,
    next_cue_id: u64,
    pub workspace_layout: WorkspaceLayout,
    pub stacked_split: Entity<ResizableState>,
    pub side_split: Entity<ResizableState>,
    pub theme: YttmTheme,
    pub input: YttmInput,
}

impl YttmApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut next_cue_id = 1;
        let cues = sample_cues(&mut next_cue_id);

        let subtitle_input = cx.new(|cx| InputState::new(window, cx).placeholder("Subtitle text"));

        cx.subscribe(&subtitle_input, |this, state, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change)
                && let Some(id) = this.selected_cue
            {
                let text = state.read(cx).value().to_string();
                if let Some(cue) = this.document.cues.iter_mut().find(|cue| cue.id == id) {
                    cue.text = text;
                    cx.notify();
                }
            }
        })
        .detach();

        Self {
            document: Document { cues },
            selected_cue: None,
            next_cue_id,
            workspace_layout: WorkspaceLayout::Stacked,
            stacked_split: cx.new(|_| ResizableState::default()),
            side_split: cx.new(|_| ResizableState::default()),
            theme: YttmTheme::new(),
            input: YttmInput {
                subtitle: subtitle_input,
            },
        }
    }

    pub fn sync_subtitle_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self
            .selected_cue
            .and_then(|id| self.document.cues.iter().find(|cue| cue.id == id))
            .map(|cue| cue.text.clone())
            .unwrap_or_default();

        self.input.subtitle.update(cx, |input, cx| {
            input.set_value(text, window, cx);
        })
    }

    /// Append a cue entry at the end
    pub fn append_cue(&mut self) {
        let start_ms = self.document.cues.last().map(|cue| cue.end_ms).unwrap_or(0);
        self.insert_cue(start_ms, start_ms + 2_000);
    }

    pub fn add_cue_above_selected(&mut self) {
        let Some(selected) = self.selected_cue() else {
            return;
        };

        let end_ms = selected.start_ms;
        let start_ms = end_ms.saturating_sub(2_000);
        self.insert_cue(start_ms, end_ms);
    }

    pub fn add_cue_below_selected(&mut self) {
        let Some(selected) = self.selected_cue() else {
            return;
        };

        let start_ms = selected.end_ms;
        self.insert_cue(start_ms, start_ms + 2_000);
    }

    pub fn delete_selected_cue(&mut self) {
        let Some(selected_id) = self.selected_cue else {
            return;
        };
        let Some(ix) = self.selected_index() else {
            self.selected_cue = None;
            return;
        };

        self.document.cues.remove(ix);

        self.selected_cue = if self.document.cues.is_empty() {
            // Empty -> Unselect
            None
        } else if ix == 0 {
            // First entry -> keep first entry
            Some(self.document.cues[0].id)
        } else {
            // Non-first entry -> to above entry
            Some(self.document.cues[ix - 1].id)
        };
        let _ = selected_id;
    }

    pub fn run() {
        gpui_kit::application()
            .with_assets(gpui_kit::assets::AllAssets)
            .run(|cx| {
                gpui_kit::init(cx);

                cx.text_system()
                    .add_fonts(vec![
                        Cow::Borrowed(include_bytes!(
                            "../../../assets/fonts/Inter/Inter-VariableFont_opsz,wght.ttf"
                        )),
                        Cow::Borrowed(include_bytes!(
                            "../../../assets/fonts/Inter/Inter-Italic-VariableFont_opsz,wght.ttf"
                        )),
                    ])
                    .expect("failed to load bundled UI font");

                Theme::update(cx, |theme| {
                    theme.font_family = "Inter".into();
                });

                gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                    cx.new(|cx| YttmApp::new(window, cx))
                })
                .expect("failed to open window");
            });
    }

    fn insert_cue(&mut self, start_ms: u64, end_ms: u64) {
        let id = self.next_cue_id;
        self.next_cue_id += 1;

        self.document.cues.push(Cue {
            id,
            start_ms,
            end_ms,
            text: String::new(),
            pos_x: None,
            pos_y: None,
        });
        self.sort_cues();
        self.selected_cue = Some(id);
    }

    fn sort_cues(&mut self) {
        self.document.cues.sort_by_key(|cue| (cue.start_ms, cue.id));
    }

    fn selected_cue(&self) -> Option<&Cue> {
        let id = self.selected_cue?;
        self.document.cues.iter().find(|cue| cue.id == id)
    }

    fn selected_index(&self) -> Option<usize> {
        let id = self.selected_cue?;
        self.document.cues.iter().position(|cue| cue.id == id)
    }
}

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
            let pill = match handle.axis() {
                Axis::Horizontal => div()
                    .flex_none()
                    .rounded(cx.theme().radius_full())
                    .bg(cx.theme().muted_foreground)
                    .opacity(opacity)
                    .w(px(4.))
                    .h(length)
                    .ml(px(4.) * -0.5),
                Axis::Vertical => div()
                    .flex_none()
                    .rounded(cx.theme().radius_full())
                    .bg(cx.theme().muted_foreground)
                    .opacity(opacity)
                    .h(px(4.))
                    .w(length)
                    .mt(px(4.) * -0.5),
            };
            line = line.child(pill);
        }

        Some(line.into_any_element())
    })
}

impl Render for YttmApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = match self.workspace_layout {
            WorkspaceLayout::Stacked => v_resizable("main-split-stacked")
                .with_state(&self.stacked_split)
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
                .child(resizable_panel().child(
                    div().size_full().flex().flex_col().pt(px(4.)).child(
                        cue_editor::render_editor_area(
                            &self.document.cues,
                            self.selected_cue,
                            &self.input.subtitle,
                            cx,
                        ),
                    ),
                )),
            WorkspaceLayout::SideBySide => h_resizable("main-split-side")
                .with_state(&self.side_split)
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
                .child(
                    resizable_panel().child(div().size_full().flex().pl(px(4.)).child(
                        cue_editor::render_editor_area(
                            &self.document.cues,
                            self.selected_cue,
                            &self.input.subtitle,
                            cx,
                        ),
                    )),
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
        let (layout_tooltip, layout_icon) = match self.workspace_layout {
            WorkspaceLayout::Stacked => ("Switch to Side by side", IconName::PanelBottom),
            WorkspaceLayout::SideBySide => ("Switch to Stacked", IconName::PanelRight),
        };

        let (theme_tooltip, theme_icon) = if cx.theme().mode.is_dark() {
            ("Switch to light theme", IconName::Moon)
        } else {
            ("Switch to dark theme", IconName::Sun)
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
            .flex_1()
            .min_h_0()
            .min_w_0()
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

fn sample_cues(next_cue_id: &mut u64) -> Vec<Cue> {
    let mut cues = vec![
        Cue {
            id: 0,
            start_ms: 1_000,
            end_ms: 3_500,
            text: "First line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 4_000,
            end_ms: 6_200,
            text: "Second line, positioned".into(),
            pos_x: Some(50),
            pos_y: Some(80),
        },
        Cue {
            id: 0,
            start_ms: 7_000,
            end_ms: 9_000,
            text: "Third line".into(),
            pos_x: Some(10),
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 9_000,
            end_ms: 11_000,
            text: "Fourth line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 15_000,
            end_ms: 20_000,
            text: "Fifth line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 21_000,
            end_ms: 23_000,
            text: "Sixth line".into(),
            pos_x: None,
            pos_y: None,
        },
    ];

    for cue in &mut cues {
        cue.id = *next_cue_id;
        *next_cue_id += 1;
    }

    cues
}
