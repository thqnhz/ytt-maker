use gpui_kit::component::input::*;
use gpui_kit::*;

#[derive(Clone, Debug, Default)]
pub struct SelectedCueValues {
    pub subtitle: String,
    pub start: String,
    pub end: String,
    pub pos_x: String,
    pub pos_y: String,
}

pub struct YttmInput {
    pub subtitle: Entity<InputState>,
    pub start: Entity<InputState>,
    pub end: Entity<InputState>,
    pub pos_x: Entity<InputState>,
    pub pos_y: Entity<InputState>,
}

impl YttmInput {
    pub fn new<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Self {
        Self {
            subtitle: cx.new(|cx| InputState::new(window, cx).placeholder("Subtitle text")),
            start: cx.new(|cx| InputState::new(window, cx).placeholder("00:00.000")),
            end: cx.new(|cx| InputState::new(window, cx).placeholder("00:00.000")),
            pos_x: cx.new(|cx| InputState::new(window, cx).placeholder("-")),
            pos_y: cx.new(|cx| InputState::new(window, cx).placeholder("-")),
        }
    }

    pub fn sync_selected<V: 'static>(
        &self,
        values: SelectedCueValues,
        window: &mut Window,
        cx: &mut Context<V>,
    ) {
        self.subtitle
            .update(cx, |i, cx| i.set_value(values.subtitle, window, cx));
        self.start
            .update(cx, |i, cx| i.set_value(values.start, window, cx));
        self.end
            .update(cx, |i, cx| i.set_value(values.end, window, cx));
        self.pos_x
            .update(cx, |i, cx| i.set_value(values.pos_x, window, cx));
        self.pos_y
            .update(cx, |i, cx| i.set_value(values.pos_y, window, cx));
    }
}

pub fn parse_ms(s: &str) -> Option<u64> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(':').collect();
    let (hours, mins, secs_part) = match parts.as_slice() {
        [m, sec] => (0u64, m.parse::<u64>().ok()?, *sec),
        [h, m, sec] => (h.parse::<u64>().ok()?, m.parse::<u64>().ok()?, *sec),
        _ => return None,
    };
    if mins >= 60 {
        return None;
    }

    let (secs, millis) = if let Some((sec_s, frac_s)) = secs_part.split_once('.') {
        let secs = sec_s.parse::<u64>().ok()?;
        if secs >= 60 {
            return None;
        }
        let mut frac: String = frac_s.chars().take(3).collect();
        while frac.len() < 3 {
            frac.push('0');
        }
        (secs, frac.parse::<u64>().ok()?)
    } else {
        let secs = secs_part.parse::<u64>().ok()?;
        if secs >= 60 {
            return None;
        }
        (secs, 0)
    };

    Some(hours * 3_600_000 + mins * 60_000 + secs * 1000 + millis)
}

pub fn parse_opt_i32(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.is_empty() || s == "-" {
        None
    } else {
        s.parse::<i32>().ok()
    }
}
