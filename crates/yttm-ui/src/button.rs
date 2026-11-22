use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::app::YttmApp;
use crate::theme::theme_toggle_icon;
use crate::theme::theme_toggle_tooltip;
use crate::workspace::Workspace;
use crate::workspace::WorkspaceLayout;

pub fn add_cue_button(cx: &Context<YttmApp>) -> Button {
    Button::new("add-cue")
        .icon(IconName::Plus)
        .ghost()
        .tooltip("Add subtitle")
        .on_click(cx.listener(|this, _, window, cx| {
            this.append_cue(window, cx);
        }))
}

pub fn add_cue_above_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("add-cue-above")
        .icon(IconName::ArrowUp)
        .ghost()
        .tooltip("Add subtitle above selection")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.add_cue_above_selected(window, cx);
        }))
}

pub fn add_cue_below_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("add-cue-below")
        .icon(IconName::ArrowDown)
        .ghost()
        .tooltip("Add subtitle below selection")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.add_cue_below_selected(window, cx);
        }))
}

pub fn duplicate_cue_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("duplicate-cue")
        .icon(IconName::Copy)
        .ghost()
        .tooltip("Duplicate selected subtitle")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.duplicate_selected_cue(window, cx);
        }))
}

pub fn delete_cue_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("delete-cue")
        .icon(IconName::Trash)
        .ghost()
        .tooltip("Delete selected subtitle")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.delete_selected_cue(window, cx);
        }))
}

pub fn undo_button(cx: &Context<YttmApp>, disabled: bool) -> Button {
    Button::new("undo")
        .icon(IconName::Undo2)
        .ghost()
        .tooltip("Undo")
        .disabled(disabled)
        .on_click(cx.listener(|this, _, window, cx| {
            this.undo(window, cx);
        }))
}

pub fn redo_button(cx: &Context<YttmApp>, disabled: bool) -> Button {
    Button::new("redo")
        .icon(IconName::Redo2)
        .ghost()
        .tooltip("Redo")
        .disabled(disabled)
        .on_click(cx.listener(|this, _, window, cx| {
            this.redo(window, cx);
        }))
}

pub fn commit_changes_button(cx: &Context<YttmApp>) -> Button {
    Button::new("commit-cue")
        .icon(IconName::Check)
        .ghost()
        .text_color(cx.theme().success)
        .tooltip("Commit changes")
        .on_click(cx.listener(|this, _, window, cx| {
            this.commit_selected_cue_inputs(window, cx);
            window.blur(cx);
        }))
}

pub fn revert_changes_button(cx: &Context<YttmApp>) -> Button {
    Button::new("revert-cue")
        .icon(IconName::Close)
        .ghost()
        .text_color(cx.theme().danger)
        .tooltip("Revert changes")
        .on_click(cx.listener(|this, _, window, cx| {
            this.sync_selected_cue_inputs(window, cx);
            window.blur(cx);
            cx.notify();
        }))
}

pub fn open_button(cx: &Context<YttmApp>) -> Button {
    Button::new("open-file")
        .icon(IconName::FolderOpen)
        .ghost()
        .tooltip("Open video")
        .on_click(cx.listener(|this, _, _, cx| {
            this.open_video(cx);
        }))
}

pub fn save_button() -> Button {
    Button::new("save-file")
        .icon(IconName::Save)
        .ghost()
        .tooltip("Save")
        .on_click(|_, _, _| {})
}

pub fn toggle_theme_button(cx: &mut Context<YttmApp>) -> Button {
    let is_dark = cx.theme().mode.is_dark();
    let theme_tooltip = theme_toggle_tooltip(is_dark);
    let theme_icon = theme_toggle_icon(is_dark);
    Button::new("toggle-theme")
        .icon(theme_icon)
        .ghost()
        .tooltip(theme_tooltip)
        .on_click(cx.listener(|this, _, _, cx| {
            this.theme.toggle_theme(cx);
        }))
}

pub fn toggle_layout_button(layout: &WorkspaceLayout, cx: &mut Context<YttmApp>) -> Button {
    let layout_tooltip = Workspace::layout_tooltip(*layout);
    let layout_icon = Workspace::layout_icon(*layout);

    Button::new("toggle-layout")
        .icon(layout_icon)
        .ghost()
        .tooltip(layout_tooltip)
        .on_click(cx.listener(|this, _, _, cx| {
            this.workspace.switch_layout();
            cx.notify();
        }))
}
