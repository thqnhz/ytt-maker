use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::app::YttmApp;

pub fn add_cue_button(cx: &Context<YttmApp>) -> Button {
    Button::new("add-cue")
        .icon(IconName::Plus)
        .ghost()
        .tooltip("Add subtitle")
        .on_click(cx.listener(|this, _, window, cx| {
            this.append_cue();
            this.sync_selected_cue_inputs(window, cx);
            this.focus_subtitle(window, cx);
            cx.notify();
        }))
}

pub fn add_cue_above_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("add-cue-above")
        .icon(IconName::ArrowUp)
        .ghost()
        .tooltip("Add subtitle above selection")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.add_cue_above_selected();
            this.sync_selected_cue_inputs(window, cx);
            this.focus_subtitle(window, cx);
            cx.notify();
        }))
}

pub fn add_cue_below_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("add-cue-below")
        .icon(IconName::ArrowDown)
        .ghost()
        .tooltip("Add subtitle below selection")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.add_cue_below_selected();
            this.sync_selected_cue_inputs(window, cx);
            this.focus_subtitle(window, cx);
            cx.notify();
        }))
}

pub fn duplicate_cue_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("duplicate-cue")
        .icon(IconName::Copy)
        .ghost()
        .tooltip("Duplicate selected subtitle")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.duplicate_selected_cue();
            this.sync_selected_cue_inputs(window, cx);
            this.focus_subtitle(window, cx);
            cx.notify();
        }))
}

pub fn delete_cue_button(cx: &Context<YttmApp>, selected_cue: Option<u64>) -> Button {
    Button::new("delete-cue")
        .icon(IconName::Trash)
        .ghost()
        .tooltip("Delete selected subtitle")
        .disabled(selected_cue.is_none())
        .on_click(cx.listener(|this, _, window, cx| {
            this.delete_selected_cue();
            this.sync_selected_cue_inputs(window, cx);
            if this.selected_cue.is_some() {
                this.focus_subtitle(window, cx);
            }
            cx.notify();
        }))
}

pub fn commit_cue_button(cx: &Context<YttmApp>) -> Button {
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

pub fn revert_cue_button(cx: &Context<YttmApp>) -> Button {
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
