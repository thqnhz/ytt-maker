pub fn format_duration_ms(ms: u64) -> String {
    let secs = ms / 1000;
    let millis = ms % 1000;
    format!("{secs}.{millis:03}s")
}

pub fn format_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = total_secs / 3600;

    if hours > 0 {
        format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
    } else {
        format!("{mins:02}:{secs:02}.{millis:03}")
    }
}

pub fn format_opt_i32(value: Option<i32>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
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
