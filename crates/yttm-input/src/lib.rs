use gpui_kit::component::input::*;
use gpui_kit::*;

pub struct YttmInput {
    pub subtitle: Entity<InputState>,
    pub start: Entity<InputState>,
    pub end: Entity<InputState>,
}

impl YttmInput {
    pub fn new<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Self {
        Self {
            subtitle: cx.new(|cx| InputState::new(window, cx).placeholder("Subtitle text")),
            start: cx.new(|cx| InputState::new(window, cx).placeholder("00:00.000")),
            end: cx.new(|cx| InputState::new(window, cx).placeholder("00:00.000")),
        }
    }

    pub fn sync_selected<V: 'static>(
        &self,
        subtitle: String,
        start: String,
        end: String,
        window: &mut Window,
        cx: &mut Context<V>,
    ) {
        self.subtitle.update(cx, |input, cx| {
            input.set_value(subtitle, window, cx);
        });
        self.start.update(cx, |input, cx| {
            input.set_value(start, window, cx);
        });
        self.end.update(cx, |input, cx| {
            input.set_value(end, window, cx);
        });
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
