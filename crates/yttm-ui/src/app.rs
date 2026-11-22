use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::base::ResizableState;
use gpui_kit::component::select::{SearchableVec, SelectEvent, SelectState};
use gpui_kit::component::*;
use gpui_kit::*;
use image::{Frame, RgbaImage};
use smallvec::SmallVec;

use crate::theme::YttmTheme;
use crate::view::{Redo, Undo};
use crate::workspace::{Workspace, WorkspaceLayout, WorkspaceSplit};
use yttm_core::{Cue, Document};
use yttm_input::YttmInput;
use yttm_playback::{Player, PreviewQuality, VideoState};

#[derive(Clone, Debug, PartialEq, Eq)]
struct EditorSnapshot {
    document: Document,
    selected_cue: Option<u64>,
}

pub(crate) struct YttmApp {
    pub(crate) document: Document,
    pub(crate) workspace: Workspace,
    pub(crate) theme: YttmTheme,
    pub(crate) input: YttmInput,
    pub(crate) video: VideoState,
    pub(crate) player: Option<Player>,
    pub(crate) video_frame: Option<Arc<RenderImage>>,
    pub(crate) preview_quality_select: Entity<SelectState<SearchableVec<&'static str>>>,
    next_cue_id: u64,
    pub(crate) selected_cue: Option<u64>,
    pub(crate) cue_list_scroll: ScrollHandle,
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
    video_poll_generation: u64,
}

impl YttmApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let quality_items = SearchableVec::new(
            PreviewQuality::ALL
                .iter()
                .map(|q| q.label())
                .collect::<Vec<_>>(),
        );
        let preview_quality_select =
            cx.new(|cx| SelectState::new(quality_items, Some(IndexPath::new(2)), window, cx));
        cx.subscribe(&preview_quality_select, |this, _, event, cx| {
            let SelectEvent::Confirm(value) = event;
            let quality = match value.as_deref() {
                Some("Full") => PreviewQuality::Full,
                Some("1/2") => PreviewQuality::Half,
                Some("1/4") => PreviewQuality::Quarter,
                Some("1/8") => PreviewQuality::Eighth,
                _ => return,
            };
            this.set_preview_quality(quality, cx);
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
            input: YttmInput::new(window, cx),
            video: VideoState::default(),
            player: Player::new().ok(),
            video_frame: None,
            preview_quality_select,
            cue_list_scroll: ScrollHandle::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            video_poll_generation: 0,
        }
    }

    pub(crate) fn set_preview_quality(&mut self, quality: PreviewQuality, cx: &mut Context<Self>) {
        if self.video.preview_quality == quality {
            return;
        }
        self.video.preview_quality = quality;
        if let Some(player) = &self.player {
            let _ = player.set_preview_quality(quality);
        }
        cx.notify();
    }

    pub(crate) fn sync_selected_cue_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let values = self
            .selected_cue()
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

    pub(crate) fn commit_selected_cue_inputs(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.selected_cue else {
            self.sync_selected_cue_inputs(window, cx);
            return;
        };

        let subtitle = self.input.subtitle.read(cx).value().to_string();
        let start = self.input.start.read(cx).value().to_string();
        let end = self.input.end.read(cx).value().to_string();
        let pos_x = self.input.pos_x.read(cx).value().to_string();
        let pos_y = self.input.pos_y.read(cx).value().to_string();

        let Some(cue) = self.document.cues.iter().find(|c| c.id == id) else {
            self.sync_selected_cue_inputs(window, cx);
            return;
        };

        let next_start = yttm_input::parse_ms(&start).unwrap_or(cue.start_ms);
        let next_end = yttm_input::parse_ms(&end).unwrap_or(cue.end_ms);
        let next_pos_x = yttm_input::parse_opt_i32(&pos_x);
        let next_pos_y = yttm_input::parse_opt_i32(&pos_y);

        let changed = cue.text != subtitle
            || cue.start_ms != next_start
            || cue.end_ms != next_end
            || cue.pos_x != next_pos_x
            || cue.pos_y != next_pos_y;
        if !changed {
            self.sync_selected_cue_inputs(window, cx);
            return;
        }

        let before = self.snapshot();
        let start_changed = cue.start_ms != next_start;
        if let Some(cue) = self.document.cues.iter_mut().find(|c| c.id == id) {
            cue.text = subtitle;
            cue.start_ms = next_start;
            cue.end_ms = next_end;
            cue.pos_x = next_pos_x;
            cue.pos_y = next_pos_y;
        }

        if start_changed {
            self.sort_cues();
        }

        self.sync_selected_cue_inputs(window, cx);
        cx.notify();
        self.push_undo(before);
    }

    pub(crate) fn focus_subtitle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.subtitle.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    pub(crate) fn open_video(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open video".into()),
        });

        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };

            this.update(cx, |this, cx| {
                this.video_poll_generation = this.video_poll_generation.wrapping_add(1);
                this.video.open(path.clone());
                if let Some(old_frame) = this.video_frame.take() {
                    cx.drop_image(old_frame, None);
                }
                if let Some(player) = &this.player {
                    let _ = player.open(&path);
                    let _ = player.pause();
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub(crate) fn toggle_video_playback(&mut self, cx: &mut Context<Self>) {
        if self.video.path.is_none() {
            return;
        }

        let Some(player) = &self.player else {
            return;
        };

        if self.video.playing {
            self.video_poll_generation = self.video_poll_generation.wrapping_add(1);
            if player.pause().is_ok() {
                self.video.playing = false;
                cx.notify();
            }
        } else {
            self.video_poll_generation = self.video_poll_generation.wrapping_add(1);
            if player.play().is_ok() {
                self.video.playing = true;
                cx.notify();
                self.poll_video(self.video_poll_generation, cx);
            }
        }
    }

    fn poll_video(&mut self, generation: u64, cx: &mut Context<Self>) {
        if !self.video.playing || self.video_poll_generation != generation {
            return;
        }

        let Some(player) = &self.player else {
            self.video.playing = false;
            cx.notify();
            return;
        };

        let mut changed = false;
        if let Some(position_ms) = player.position_ms()
            && self.video.position_ms != position_ms
        {
            self.video.position_ms = position_ms;
            changed = true;
        }
        if let Some(duration_ms) = player.duration_ms()
            && self.video.duration_ms != Some(duration_ms)
        {
            self.video.duration_ms = Some(duration_ms);
            changed = true;
        }
        if player.has_frame_update()
            && let Some(frame) = player.latest_frame()
            && let Some(img) = RgbaImage::from_raw(frame.width, frame.height, frame.bgra)
        {
            let render = Arc::new(RenderImage::new(SmallVec::from_elem(Frame::new(img), 1)));
            if let Some(old_frame) = self.video_frame.replace(render) {
                cx.drop_image(old_frame, None);
            }
            changed = true;
        }

        if changed {
            cx.notify();
        }

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(33))
                .await;
            this.update(cx, |this, cx| this.poll_video(generation, cx))
                .ok();
        })
        .detach();
    }

    pub(crate) fn select_cue(&mut self, cue_id: u64, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_cue == Some(cue_id) {
            return;
        }
        self.selected_cue = Some(cue_id);
        self.sync_selected_cue_inputs(window, cx);
        self.focus_subtitle(window, cx);
        self.scroll_selected_into_view();
        cx.notify();
    }

    pub(crate) fn append_cue(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.snapshot();
        let start_ms = self.document.cues.last().map(|cue| cue.end_ms).unwrap_or(0);
        self.insert_cue(start_ms, start_ms + 2_000);
        self.finish_cue_mutation(before, window, cx, true);
    }

    pub(crate) fn add_cue_above_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((_, start_ms, end_ms)) = self.selected_cue().map(|cue| {
            let end_ms = cue.start_ms;
            (cue.id, end_ms.saturating_sub(2_000), end_ms)
        }) else {
            return;
        };
        let before = self.snapshot();
        self.insert_cue(start_ms, end_ms);
        self.finish_cue_mutation(before, window, cx, true);
    }

    pub(crate) fn add_cue_below_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(start_ms) = self.selected_cue().map(|cue| cue.end_ms) else {
            return;
        };
        let before = self.snapshot();
        self.insert_cue(start_ms, start_ms + 2_000);
        self.finish_cue_mutation(before, window, cx, true);
    }

    pub(crate) fn duplicate_selected_cue(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(cue) = self.selected_cue().cloned() else {
            return;
        };
        let before = self.snapshot();
        let mut new_cue = cue;
        new_cue.id = self.next_cue_id;

        self.document.cues.push(new_cue);
        self.sort_cues();
        self.selected_cue = Some(self.next_cue_id);
        self.next_cue_id += 1;
        self.finish_cue_mutation(before, window, cx, true);
    }

    pub(crate) fn delete_selected_cue(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self.selected_index() else {
            if self.selected_cue.take().is_some() {
                self.sync_selected_cue_inputs(window, cx);
                cx.notify();
            }
            return;
        };
        let before = self.snapshot();

        self.document.cues.remove(ix);

        self.selected_cue = if self.document.cues.is_empty() {
            None
        } else if ix == 0 {
            Some(self.document.cues[0].id)
        } else {
            Some(self.document.cues[ix - 1].id)
        };
        self.finish_cue_mutation(before, window, cx, self.selected_cue.is_some());
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub(crate) fn undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(snapshot) = self.undo_stack.pop() else {
            return;
        };
        self.redo_stack.push(self.snapshot());
        self.restore_snapshot(snapshot, window, cx);
    }

    pub(crate) fn redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub(crate) fn selected_cue(&self) -> Option<&Cue> {
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
        if self.document != before.document || self.selected_cue != before.selected_cue {
            self.undo_stack.push(before);
            self.redo_stack.clear();
            const MAX_UNDO: usize = 200;
            if self.undo_stack.len() > MAX_UNDO {
                let overflow = self.undo_stack.len() - MAX_UNDO;
                self.undo_stack.drain(0..overflow);
            }
        }
    }

    fn finish_cue_mutation(
        &mut self,
        before: EditorSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
        focus_subtitle: bool,
    ) {
        self.sync_selected_cue_inputs(window, cx);
        if focus_subtitle {
            self.focus_subtitle(window, cx);
        }
        self.scroll_selected_into_view();
        cx.notify();
        self.push_undo(before);
    }

    fn scroll_selected_into_view(&self) {
        if let Some(ix) = self.selected_index() {
            self.cue_list_scroll.scroll_to_item(ix);
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
        self.scroll_selected_into_view();
        cx.notify();
    }
}
