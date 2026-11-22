use std::borrow::Cow;

use gpui_kit::base::ResizableState;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::theme::YttmTheme;
use crate::view::{Redo, Undo};
use crate::workspace::{Workspace, WorkspaceLayout, WorkspaceSplit};
use yttm_core::{Cue, Document};
use yttm_input::YttmInput;

#[derive(Clone, Debug, PartialEq, Eq)]
struct EditorSnapshot {
    document: Document,
    selected_cue: Option<u64>,
}

pub struct YttmApp {
    pub document: Document,
    pub workspace: Workspace,
    pub theme: YttmTheme,
    pub input: YttmInput,
    next_cue_id: u64,
    pub(crate) selected_cue: Option<u64>,
    pub(crate) cue_list_scroll: ScrollHandle,
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
}

impl YttmApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
            input: YttmInput::new(window, cx),
            cue_list_scroll: ScrollHandle::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
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
                pos_x: cue.pos_x.map(|v| v.to_string()).unwrap_or_default(),
                pos_y: cue.pos_y.map(|v| v.to_string()).unwrap_or_default(),
            })
            .unwrap_or_default();

        self.input.sync_selected(values, window, cx);
    }

    pub fn commit_selected_cue_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.snapshot();
        let Some(id) = self.selected_cue else {
            self.sync_selected_cue_inputs(window, cx);
            return;
        };

        let subtitle = self.input.subtitle.read(cx).value().to_string();
        let start = self.input.start.read(cx).value().to_string();
        let end = self.input.end.read(cx).value().to_string();
        let pos_x = self.input.pos_x.read(cx).value().to_string();
        let pos_y = self.input.pos_y.read(cx).value().to_string();

        let mut should_sort = false;
        if let Some(cue) = self.document.cues.iter_mut().find(|c| c.id == id) {
            cue.text = subtitle;

            if let Some(ms) = yttm_input::parse_ms(&start) {
                cue.start_ms = ms;
                should_sort = true;
            }
            if let Some(ms) = yttm_input::parse_ms(&end) {
                cue.end_ms = ms;
            }

            cue.pos_x = yttm_input::parse_opt_i32(&pos_x);
            cue.pos_y = yttm_input::parse_opt_i32(&pos_y);
        }

        if should_sort {
            self.sort_cues();
        }

        self.sync_selected_cue_inputs(window, cx);
        cx.notify();
        self.push_undo(before);
    }

    pub fn focus_subtitle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.subtitle.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    /// Append a cue entry at the end
    pub fn append_cue(&mut self) {
        let before = self.snapshot();
        let start_ms = self.document.cues.last().map(|cue| cue.end_ms).unwrap_or(0);
        self.insert_cue(start_ms, start_ms + 2_000);
        self.push_undo(before);
    }

    pub fn add_cue_above_selected(&mut self) {
        let Some(selected) = self.selected_cue() else {
            return;
        };
        let before = self.snapshot();

        let end_ms = selected.start_ms;
        let start_ms = end_ms.saturating_sub(2_000);
        self.insert_cue(start_ms, end_ms);
        self.push_undo(before);
    }

    pub fn add_cue_below_selected(&mut self) {
        let Some(selected) = self.selected_cue() else {
            return;
        };
        let before = self.snapshot();

        let start_ms = selected.end_ms;
        self.insert_cue(start_ms, start_ms + 2_000);
        self.push_undo(before);
    }

    pub fn duplicate_selected_cue(&mut self) {
        let Some(id) = self.selected_cue else {
            return;
        };
        let Some(cue) = self.document.cues.iter().find(|c| c.id == id) else {
            return;
        };
        let before = self.snapshot();
        let mut new_cue = cue.clone();
        new_cue.id = self.next_cue_id;

        self.document.cues.push(new_cue);
        self.sort_cues();
        self.selected_cue = Some(self.next_cue_id);
        self.next_cue_id += 1;
        self.push_undo(before);
    }

    pub fn delete_selected_cue(&mut self) {
        if self.selected_cue.is_none() {
            return;
        }
        let Some(ix) = self.selected_index() else {
            self.selected_cue = None;
            return;
        };
        let before = self.snapshot();

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
        self.push_undo(before);
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(snapshot) = self.undo_stack.pop() else {
            return;
        };
        self.redo_stack.push(self.snapshot());
        self.restore_snapshot(snapshot, window, cx);
    }

    pub fn redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(snapshot) = self.redo_stack.pop() else {
            return;
        };
        self.undo_stack.push(self.snapshot());
        self.restore_snapshot(snapshot, window, cx);
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

                // Global keybind
                cx.bind_keys([
                    KeyBinding::new("ctrl-z", Undo, None),
                    KeyBinding::new("ctrl-y", Redo, None),
                ]);

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

    fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            document: self.document.clone(),
            selected_cue: self.selected_cue,
        }
    }

    fn push_undo(&mut self, before: EditorSnapshot) {
        if self.snapshot() != before {
            self.undo_stack.push(before);
            self.redo_stack.clear();
        }
    }

    fn restore_snapshot(
        &mut self,
        snapshot: EditorSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.document = snapshot.document;
        self.selected_cue = snapshot.selected_cue;
        self.sync_selected_cue_inputs(window, cx);
        cx.notify();
    }
}
