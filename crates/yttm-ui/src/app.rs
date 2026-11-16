use std::borrow::Cow;

use gpui_kit::base::ResizableState;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::*;
use gpui_kit::*;

use crate::theme::YttmTheme;
use crate::workspace::{Workspace, WorkspaceLayout, WorkspaceSplit};
use yttm_core::{Cue, Document};
use yttm_input::YttmInput;

pub struct YttmApp {
    pub document: Document,
    pub workspace: Workspace,
    pub theme: YttmTheme,
    pub input: YttmInput,
    next_cue_id: u64,
    pub(crate) selected_cue: Option<u64>,
}

impl YttmApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
            document: Document { cues: Vec::new() },
            next_cue_id: 1,
            selected_cue: None,
            workspace: Workspace {
                layout: WorkspaceLayout::Stacked,
                split: WorkspaceSplit {
                    stacked: cx.new(|_| ResizableState::default()),
                    side_by_side: cx.new(|_| ResizableState::default()),
                },
            },
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
