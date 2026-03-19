use crate::data::languages::LanguageBreakdown;
use crate::display::theme::ThemeColors;
use crate::lang_colors;
use owo_colors::OwoColorize;

/// Render a proportional color bar and legend lines for languages.
pub fn render_bar(
    langs: &LanguageBreakdown,
    bar_width: usize,
    colors: &ThemeColors,
    no_color: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    if langs.entries.is_empty() {
        return lines;
    }

    // Build the bar
    let mut bar = String::new();
    let mut used = 0usize;

    for (i, entry) in langs.entries.iter().enumerate() {
        let width = if i == langs.entries.len() - 1 {
            bar_width - used
        } else {
            let w = ((entry.percentage / 100.0) * bar_width as f64).round() as usize;
            w.max(1).min(bar_width - used)
        };

        if width == 0 {
            continue;
        }

        let segment = "█".repeat(width);
        if no_color {
            bar.push_str(&segment);
        } else {
            let (r, g, b) = lang_colors::get_color(&entry.name);
            bar.push_str(&format!("{}", segment.truecolor(r, g, b)));
        }
        used += width;
    }

    lines.push(bar);

    // Build legend lines (fit multiple entries per line)
    let mut legend_line = String::new();
    let mut legend_visible_len = 0;
    let max_legend_width = bar_width;

    for entry in &langs.entries {
        let (r, g, b) = lang_colors::get_color(&entry.name);
        let dot = if no_color {
            "●".to_string()
        } else {
            format!("{}", "●".truecolor(r, g, b))
        };

        let label = format!("{} {:.1}%", entry.name, entry.percentage);
        let entry_text = if no_color {
            format!("{dot} {label}")
        } else {
            format!("{dot} {}", colors.value(&label))
        };

        let entry_visible_len = 2 + entry.name.len() + 1 + format!("{:.1}%", entry.percentage).len();

        if legend_visible_len > 0 && legend_visible_len + 2 + entry_visible_len > max_legend_width {
            lines.push(legend_line);
            legend_line = String::new();
            legend_visible_len = 0;
        }

        if legend_visible_len > 0 {
            legend_line.push_str("  ");
            legend_visible_len += 2;
        }

        legend_line.push_str(&entry_text);
        legend_visible_len += entry_visible_len;
    }

    if !legend_line.is_empty() {
        lines.push(legend_line);
    }

    lines
}
