use crate::cli::{OrgOpts, Theme};
use crate::data::org::OrgProfile;
use crate::display::theme::ThemeColors;
use crate::display::*;

pub fn render(profile: &OrgProfile, opts: &OrgOpts, theme: Theme, no_color: bool) {
    let colors = ThemeColors::from_theme(theme);

    if opts.detailed_languages() {
        if let Some(ref langs) = profile.languages {
            language_bar::render_detail_table(langs, &profile.login, &colors, no_color);
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
        format!("{} @ GitHub (org)", profile.login)
    } else {
        format!(
            "{} {} {}",
            colors.title(&profile.login),
            colors.muted("@"),
            colors.muted("GitHub (org)")
        )
    };
    lines.push(title);

    let sep = "─".repeat(profile.login.len() + 16);
    lines.push(if no_color { sep } else { colors.muted(&sep) });

    // Fields
    let label_w = 12;
    let fields: Vec<(&str, Option<String>)> = vec![
        ("Name", profile.name.clone().filter(|s| !s.is_empty())),
        (
            "Description",
            profile.description.clone().filter(|s| !s.is_empty()),
        ),
        (
            "Location",
            profile.location.clone().filter(|s| !s.is_empty()),
        ),
        ("Blog", profile.blog.clone().filter(|s| !s.is_empty())),
        ("Twitter", profile.twitter.clone().map(|t| format!("@{t}"))),
        ("Repos", Some(format_number(profile.public_repos))),
        ("Followers", Some(format_number(profile.followers))),
        ("Created", Some(profile.created.clone())),
    ];

    for (label, value) in &fields {
        if let Some(val) = value {
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
    }

    // Aggregate stats
    lines.push(String::new());
    let stars = if no_color {
        format!(
            "{:<label_w$}{:<10}{:<label_w$}{}",
            "Stars:",
            format_number(profile.stats.total_stars),
            "Forks:",
            format_number(profile.stats.total_forks)
        )
    } else {
        format!(
            "{}{:<10}{}{}",
            colors.label(&format!("{:<label_w$}", "Stars:")),
            format_number(profile.stats.total_stars),
            colors.label(&format!("{:<label_w$}", "Forks:")),
            colors.value(&format_number(profile.stats.total_forks))
        )
    };
    lines.push(stars);

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

    // Top repos
    if (opts.show_repos() || opts.show_full_card())
        && let Some(ref repos) = profile.top_repos
        && !repos.is_empty()
    {
        lines.push(String::new());
        lines.push(if no_color {
            "Top Repositories".to_string()
        } else {
            colors.title("Top Repositories")
        });

        for r in repos {
            let lang = r.language.as_deref().unwrap_or("");
            let line = if no_color {
                let private_tag = if r.is_private { " 🔒" } else { "" };
                format!(
                    "  {}{private_tag} ★{} ⑂{}  {lang}",
                    r.name,
                    format_number(r.stars),
                    format_number(r.forks)
                )
            } else {
                format!(
                    "  {}{} {}{}  {}",
                    colors.accent(&r.name),
                    if r.is_private {
                        colors.muted(" 🔒")
                    } else {
                        String::new()
                    },
                    colors.value(&format!("★{}", format_number(r.stars))),
                    colors.muted(&format!(" ⑂{}", format_number(r.forks))),
                    colors.muted(lang)
                )
            };
            lines.push(line);
        }
    }

    // Render
    println!("{}", top_border(width, &colors, no_color));
    for line in &lines {
        println!("{}", card_line(line, width, &colors, no_color));
    }
    println!("{}", bottom_border(width, &colors, no_color));
}
