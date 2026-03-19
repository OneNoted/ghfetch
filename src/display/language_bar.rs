use crate::data::languages::{self, LanguageBreakdown};
use crate::display::theme::ThemeColors;
use crate::lang_colors;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
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

    // Table
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(if no_color {
            comfy_table::presets::ASCII_BORDERS_ONLY
        } else {
            comfy_table::presets::UTF8_BORDERS_ONLY
        });

    table.set_header(vec![
        Cell::new("Language"),
        Cell::new("Bytes").set_alignment(CellAlignment::Right),
        Cell::new("Lines (est.)").set_alignment(CellAlignment::Right),
        Cell::new("%").set_alignment(CellAlignment::Right),
        Cell::new("Repos").set_alignment(CellAlignment::Right),
    ]);

    for entry in &langs.entries {
        let (r, g, b) = lang_colors::get_color(&entry.name);
        let dot_name = if no_color {
            format!("● {}", entry.name)
        } else {
            format!("{} {}", "●".truecolor(r, g, b), entry.name)
        };

        table.add_row(vec![
            Cell::new(dot_name),
            Cell::new(languages::format_bytes(entry.bytes)).set_alignment(CellAlignment::Right),
            Cell::new(languages::format_lines(entry.bytes)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.1}%", entry.percentage)).set_alignment(CellAlignment::Right),
            Cell::new(entry.repo_count.to_string()).set_alignment(CellAlignment::Right),
        ]);
    }

    // Totals row
    table.add_row(vec![
        Cell::new("Total").add_attribute(Attribute::Bold),
        Cell::new(languages::format_bytes(langs.total_bytes))
            .set_alignment(CellAlignment::Right)
            .add_attribute(Attribute::Bold),
        Cell::new(languages::format_lines(langs.total_bytes))
            .set_alignment(CellAlignment::Right)
            .add_attribute(Attribute::Bold),
        Cell::new("100.0%")
            .set_alignment(CellAlignment::Right)
            .add_attribute(Attribute::Bold),
        Cell::new(""),
    ]);

    println!("{table}");
}
