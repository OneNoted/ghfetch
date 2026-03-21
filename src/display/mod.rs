pub mod language_bar;
pub mod org;
pub mod repo;
pub mod streak;
pub mod theme;
pub mod user;

use theme::ThemeColors;
use unicode_width::UnicodeWidthChar;

/// Card-rendering helpers shared across display modules.
pub fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}

pub fn card_width() -> usize {
    terminal_width().clamp(45, 60)
}

pub fn top_border(width: usize, colors: &ThemeColors, no_color: bool) -> String {
    let inner = "─".repeat(width - 2);
    let line = format!("╭{inner}╮");
    if no_color {
        line
    } else {
        colors.border(&line)
    }
}

pub fn bottom_border(width: usize, colors: &ThemeColors, no_color: bool) -> String {
    let inner = "─".repeat(width - 2);
    let line = format!("╰{inner}╯");
    if no_color {
        line
    } else {
        colors.border(&line)
    }
}

pub fn card_line(content: &str, width: usize, colors: &ThemeColors, no_color: bool) -> String {
    // content is the inner text (without borders). Pad to fill width.
    let inner_width = width - 4; // "│ " + content + " │"
    let fitted = truncate_ansi(content, inner_width);
    let visible_len = strip_ansi_len(&fitted);
    let padding = if visible_len < inner_width {
        " ".repeat(inner_width - visible_len)
    } else {
        String::new()
    };

    let border_l = if no_color {
        "│".to_string()
    } else {
        colors.border("│")
    };
    let border_r = if no_color {
        "│".to_string()
    } else {
        colors.border("│")
    };

    format!("{border_l} {fitted}{padding} {border_r}")
}

#[allow(dead_code)]
pub fn empty_line(width: usize, colors: &ThemeColors, no_color: bool) -> String {
    card_line("", width, colors, no_color)
}

#[allow(dead_code)]
pub fn separator_line(width: usize, colors: &ThemeColors, no_color: bool) -> String {
    let inner_width = width - 4;
    let sep = "─".repeat(inner_width);
    let content = if no_color {
        sep
    } else {
        colors.muted(&sep)
    };
    card_line(&content, width, colors, no_color)
}

/// Estimate visible character length by stripping ANSI escape sequences.
fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0usize;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += UnicodeWidthChar::width(c).unwrap_or(0);
        }
    }
    len
}

fn truncate_ansi(s: &str, max_width: usize) -> String {
    const ELLIPSIS: &str = "...";
    let ellipsis_width = ELLIPSIS.len();

    if strip_ansi_len(s) <= max_width {
        return s.to_string();
    }

    let target_width = max_width.saturating_sub(ellipsis_width);
    let mut out = String::new();
    let mut width = 0usize;
    let mut saw_ansi = false;
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            saw_ansi = true;
            out.push(c);
            for next in chars.by_ref() {
                out.push(next);
                if next == 'm' {
                    break;
                }
            }
            continue;
        }

        let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
        let limit = if max_width > ellipsis_width {
            target_width
        } else {
            max_width
        };

        if width + char_width > limit {
            break;
        }

        out.push(c);
        width += char_width;
    }

    if max_width > ellipsis_width {
        if saw_ansi {
            out.push_str("\x1b[0m");
        }
        out.push_str(ELLIPSIS);
    }

    out
}

/// Format a number with commas: 12345 → "12,345"
pub fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
