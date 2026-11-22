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
    value.map(|v| v.to_string()).unwrap_or_default()
}
