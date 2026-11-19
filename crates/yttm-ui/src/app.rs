use std::borrow::Cow;

use gpui_kit::base::ResizableState;
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
        let input = YttmInput::new(window, cx);

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
            input,
        }
    }

    pub fn sync_selected_cue_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let values = self
            .selected_cue
            .and_then(|id| self.document.cues.iter().find(|cue| cue.id == id))
            .map(|cue| yttm_input::SelectedCueValues {
                subtitle: cue.text.clone(),
                start: crate::format::format_ms(cue.start_ms),
                end: crate::format::format_ms(cue.end_ms),
                pos_x: crate::format::format_opt_i32(cue.pos_x),
                pos_y: crate::format::format_opt_i32(cue.pos_y),
            })
            .unwrap_or_default();

        self.input.sync_selected(values, window, cx);
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

    pub fn duplicate_selected_cue(&mut self) {
        let Some(id) = self.selected_cue else {
            return;
        };
        let Some(cue) = self.document.cues.iter().find(|c| c.id == id) else {
            return;
        };

        let text = cue.text.clone();
        let pos_x = cue.pos_x;
        let pos_y = cue.pos_y;
        let start_ms = cue.start_ms;
        let end_ms = cue.end_ms;

        let new_id = self.next_cue_id;
        self.next_cue_id += 1;

        self.document.cues.push(Cue {
            id: new_id,
            start_ms,
            end_ms,
            text,
            pos_x,
            pos_y,
        });
        self.sort_cues();
        self.selected_cue = Some(new_id);
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

    pub(crate) fn sort_cues(&mut self) {
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
