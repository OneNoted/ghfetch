use crate::data::contributions::StreakInfo;
use crate::display::theme::ThemeColors;

pub fn render_streaks(streaks: &StreakInfo, colors: &ThemeColors, no_color: bool) -> Vec<String> {
    let mut lines = Vec::new();

    let current = if streaks.current_streak > 0 {
        let range = match (&streaks.current_start, &streaks.current_end) {
            (Some(s), Some(e)) => format!(" ({s} – {e})"),
            _ => String::new(),
        };
        format!("{} days{range}", streaks.current_streak)
    } else {
        "0 days".to_string()
    };

    let longest = if streaks.longest_streak > 0 {
        let range = match (&streaks.longest_start, &streaks.longest_end) {
            (Some(s), Some(e)) => format!(" ({s} – {e})"),
            _ => String::new(),
        };
        format!("{} days{range}", streaks.longest_streak)
    } else {
        "0 days".to_string()
    };

    let label_w = 9; // "Current: " or "Longest: "
    if no_color {
        lines.push(format!("{:<label_w$}{current}", "Current:"));
        lines.push(format!("{:<label_w$}{longest}", "Longest:"));
    } else {
        lines.push(format!(
            "{}{}",
            colors.label(&format!("{:<label_w$}", "Current:")),
            colors.accent(&current)
        ));
        lines.push(format!(
            "{}{}",
            colors.label(&format!("{:<label_w$}", "Longest:")),
            colors.value(&longest)
        ));
    }

    lines
}
