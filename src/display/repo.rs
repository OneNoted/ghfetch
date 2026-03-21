use crate::cli::{RepoOpts, Theme};
use crate::data::repo::RepoProfile;
use crate::display::theme::ThemeColors;
use crate::display::*;

pub fn render(profile: &RepoProfile, opts: &RepoOpts, theme: Theme, no_color: bool) {
    let colors = ThemeColors::from_theme(theme);

    if opts.detailed_languages() {
        if let Some(ref langs) = profile.languages {
            language_bar::render_detail_table(langs, &profile.full_name, &colors, no_color);
        } else {
            println!("No language data available.");
        }
        return;
    }

    let width = card_width();
    let inner = width - 4;

    let mut lines = Vec::new();

    // Title
    let title = if no_color {
        format!("{} @ GitHub", profile.full_name)
    } else {
        format!(
            "{}{}{} {} {}",
            colors.muted(&profile.owner),
            colors.muted("/"),
            colors.title(&profile.name),
            colors.muted("@"),
            colors.muted("GitHub")
        )
    };
    lines.push(title);

    let sep = "─".repeat(profile.full_name.len() + 9);
    lines.push(if no_color { sep } else { colors.muted(&sep) });

    // Description
    if let Some(ref desc) = profile.description {
        lines.push(if no_color {
            desc.clone()
        } else {
            colors.value(desc)
        });
        lines.push(String::new());
    }

    // Fields
    let label_w = 12;
    let fields: Vec<(&str, String)> = vec![
        ("Stars", format_number(profile.stars)),
        ("Forks", format_number(profile.forks)),
        ("Watchers", format_number(profile.watchers)),
        ("Issues", format_number(profile.open_issues)),
        ("Size", format!("{} KB", format_number(profile.size_kb))),
        ("Branch", profile.default_branch.clone()),
        (
            "License",
            profile.license.as_deref().unwrap_or("None").to_string(),
        ),
        ("Created", profile.created.clone()),
        ("Updated", profile.updated.clone()),
    ];

    for (label, val) in &fields {
        let formatted = if no_color {
            format!("{:<label_w$}{val}", format!("{label}:"))
        } else {
            format!(
                "{}{}",
                colors.label(&format!("{:<label_w$}", format!("{label}:"))),
                colors.value(val)
            )
        };
        lines.push(formatted);
    }

    if profile.archived {
        lines.push(if no_color {
            "[ARCHIVED]".to_string()
        } else {
            colors.muted("[ARCHIVED]")
        });
    }

    if !profile.topics.is_empty() {
        lines.push(String::new());
        let topics = profile.topics.join(", ");
        let topic_line = if no_color {
            format!("Topics: {topics}")
        } else {
            format!("{}{}", colors.label("Topics: "), colors.muted(&topics))
        };
        lines.push(topic_line);
    }

    // Languages
    if opts.show_languages()
        && let Some(ref langs) = profile.languages
        && !langs.entries.is_empty()
    {
        lines.push(String::new());
        lines.push(if no_color {
            "Languages".to_string()
        } else {
            colors.title("Languages")
        });

        let bar_lines = language_bar::render_bar(langs, inner.min(40), &colors, no_color);
        lines.extend(bar_lines);
    }

    // Render
    println!("{}", top_border(width, &colors, no_color));
    for line in &lines {
        println!("{}", card_line(line, width, &colors, no_color));
    }
    println!("{}", bottom_border(width, &colors, no_color));
}
