use crate::data::languages::{self, LanguageBreakdown};
use crate::display::theme::ThemeColors;
use crate::lang_colors;
use owo_colors::OwoColorize;

/// Render a proportional color bar and legend lines for languages.
/// Used inside the card view.
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

/// Render a detailed language breakdown table.
/// Printed directly to stdout (outside the card).
pub fn render_detail_table(
    langs: &LanguageBreakdown,
    title: &str,
    colors: &ThemeColors,
    no_color: bool,
) {
    if langs.entries.is_empty() {
        println!("No language data available.");
        return;
    }

    // Header
    let header = if no_color {
        format!("{title} — Language Breakdown")
    } else {
        format!(
            "{} {} {}",
            colors.title(title),
            colors.muted("—"),
            colors.title("Language Breakdown")
        )
    };
    println!("\n{header}");

    // Bar (wider since we're outside the card)
    let bar_width = crate::display::terminal_width().clamp(40, 80);
    let bar_lines = render_bar(langs, bar_width, colors, no_color);
    for line in &bar_lines {
        println!("{line}");
    }
    println!();

    // Build table rows as plain strings, then format manually.
    // comfy-table can't measure ANSI escape widths correctly, so we avoid
    // embedding color codes in cells.
    let term_width = crate::display::terminal_width();

    // Column widths: Language(dynamic), Bytes(10), Lines(12), %(8), Repos(6)
    let fixed_cols = 10 + 12 + 8 + 6 + 8; // widths + separators/padding
    let lang_col = (term_width.saturating_sub(fixed_cols)).clamp(12, 25);
    let total_width = lang_col + fixed_cols;

    // Helper to format a row
    let fmt_row = |name: &str, bytes: &str, lines: &str, pct: &str, repos: &str| -> String {
        format!(
            "  {:<lang_col$} {:>10} {:>12} {:>8} {:>6}",
            name, bytes, lines, pct, repos
        )
    };

    // Header row
    let header_line = fmt_row("Language", "Bytes", "Lines (est.)", "%", "Repos");
    if no_color {
        println!("{header_line}");
        println!("  {}", "─".repeat(total_width - 2));
    } else {
        println!("{}", colors.label(&header_line));
        println!("  {}", colors.muted(&"─".repeat(total_width - 2)));
    }

    // Data rows
    for entry in &langs.entries {
        let (r, g, b) = lang_colors::get_color(&entry.name);
        let dot = if no_color {
            "●".to_string()
        } else {
            format!("{}", "●".truecolor(r, g, b))
        };

        // The dot is 1 visible char but may have ANSI codes, so we pad the name manually
        let name_padded = entry.name.clone();
        let bytes_str = languages::format_bytes(entry.bytes);
        let lines_str = languages::format_lines(entry.bytes);
        let pct_str = format!("{:.1}%", entry.percentage);
        let repos_str = entry.repo_count.to_string();

        // Print with the colored dot prepended (not inside the padding calculation)
        let row = format!(
            "  {dot} {:<name_w$} {:>10} {:>12} {:>8} {:>6}",
            name_padded,
            bytes_str,
            lines_str,
            pct_str,
            repos_str,
            name_w = lang_col - 3, // "● " = 2 visible chars + 1 space
        );

        if no_color {
            println!("{row}");
        } else {
            // We already have the colored dot, print as-is
            println!("{row}");
        }
    }

    // Separator + totals
    if no_color {
        println!("  {}", "─".repeat(total_width - 2));
    } else {
        println!("  {}", colors.muted(&"─".repeat(total_width - 2)));
    }

    let total_row = format!(
        "  {:<lang_col$} {:>10} {:>12} {:>8}",
        "Total",
        languages::format_bytes(langs.total_bytes),
        languages::format_lines(langs.total_bytes),
        "100.0%",
    );
    if no_color {
        println!("{total_row}");
    } else {
        println!("{}", total_row.bold());
    }

    println!();
}
