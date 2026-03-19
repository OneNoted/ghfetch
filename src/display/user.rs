use crate::cli::{Theme, UserOpts};
use crate::data::user::UserProfile;
use crate::display::*;
use crate::display::theme::ThemeColors;

pub fn render(profile: &UserProfile, opts: &UserOpts, theme: Theme, no_color: bool) {
    let colors = ThemeColors::from_theme(theme);
    let width = card_width();
    let inner = width - 4;

    let mut lines = Vec::new();

    // Title
    let title = if no_color {
        format!("{} @ GitHub", profile.login)
    } else {
        format!("{} {} {}", colors.title(&profile.login), colors.muted("@"), colors.muted("GitHub"))
    };
    lines.push(title);

    // Separator
    let sep = "─".repeat(profile.login.len() + 9);
    lines.push(if no_color { sep } else { colors.muted(&sep) });

    // Profile fields
    let fields: Vec<(&str, Option<String>)> = vec![
        ("Name", profile.name.clone()),
        ("Bio", profile.bio.clone()),
        ("Location", profile.location.clone()),
        ("Company", profile.company.clone()),
        ("Blog", profile.blog.clone().filter(|s| !s.is_empty())),
        ("Twitter", profile.twitter.clone().map(|t| format!("@{t}"))),
        ("Joined", Some(profile.joined.clone())),
        (
            "Followers",
            Some(format!(
                "{}  Following: {}",
                format_number(profile.followers),
                format_number(profile.following)
            )),
        ),
    ];

    for (label, value) in &fields {
        if let Some(val) = value {
            let formatted = if no_color {
                format!("{:<12}{}", format!("{}:", label), val)
            } else {
                format!(
                    "{}{}",
                    colors.label(&format!("{:<12}", format!("{}:", label))),
                    colors.value(val)
                )
            };
            lines.push(formatted);
        }
    }

    // Stats section
    if opts.show_contributions() || opts.show_full_card() {
        lines.push(String::new());

        let label_w = 14;

        // Stars + PRs on one line
        if let Some(ref c) = profile.contributions {
            let stats_line = if no_color {
                format!(
                    "{:<label_w$}{:<10}{:<label_w$}{}",
                    "Stars:", format_number(profile.stats.total_stars),
                    "PRs:", format_number(c.total_prs)
                )
            } else {
                format!(
                    "{}{:<10}{}{}",
                    colors.label(&format!("{:<label_w$}", "Stars:")),
                    format_number(profile.stats.total_stars),
                    colors.label(&format!("{:<label_w$}", "PRs:")),
                    colors.value(&format_number(c.total_prs))
                )
            };
            lines.push(stats_line);

            // Commits + Issues
            let stats_line2 = if no_color {
                format!(
                    "{:<label_w$}{:<10}{:<label_w$}{}",
                    "Commits:", format_number(c.total_commits),
                    "Issues:", format_number(c.total_issues)
                )
            } else {
                format!(
                    "{}{:<10}{}{}",
                    colors.label(&format!("{:<label_w$}", "Commits:")),
                    format_number(c.total_commits),
                    colors.label(&format!("{:<label_w$}", "Issues:")),
                    colors.value(&format_number(c.total_issues))
                )
            };
            lines.push(stats_line2);

            // Reviews
            let reviews = if no_color {
                format!("{:<label_w$}{}", "Reviews:", format_number(c.total_reviews))
            } else {
                format!(
                    "{}{}",
                    colors.label(&format!("{:<label_w$}", "Reviews:")),
                    colors.value(&format_number(c.total_reviews))
                )
            };
            lines.push(reviews);

            // Total contributions
            let total = if no_color {
                format!(
                    "{:<label_w$}{} (last year)",
                    "Contributions:",
                    format_number(c.total_contributions)
                )
            } else {
                format!(
                    "{}{}{}",
                    colors.label(&format!("{:<label_w$}", "Contributions:")),
                    colors.accent(&format_number(c.total_contributions)),
                    colors.muted(" (last year)")
                )
            };
            lines.push(total);
        } else {
            // Unauth: show what we have from REST
            let stars = if no_color {
                format!(
                    "{:<label_w$}{:<10}{:<label_w$}{}",
                    "Stars:", format_number(profile.stats.total_stars),
                    "Repos:", format_number(profile.stats.total_repos)
                )
            } else {
                format!(
                    "{}{:<10}{}{}",
                    colors.label(&format!("{:<label_w$}", "Stars:")),
                    format_number(profile.stats.total_stars),
                    colors.label(&format!("{:<label_w$}", "Repos:")),
                    colors.value(&format_number(profile.stats.total_repos))
                )
            };
            lines.push(stars);

            let forks = if no_color {
                format!("{:<label_w$}{}", "Forks:", format_number(profile.stats.total_forks))
            } else {
                format!(
                    "{}{}",
                    colors.label(&format!("{:<label_w$}", "Forks:")),
                    colors.value(&format_number(profile.stats.total_forks))
                )
            };
            lines.push(forks);
        }
    }

    // Languages section
    if opts.show_languages()
        && let Some(ref langs) = profile.languages
            && !langs.entries.is_empty() {
                lines.push(String::new());
                let header = if no_color {
                    "Languages".to_string()
                } else {
                    colors.title("Languages")
                };
                lines.push(header);

                let bar_lines =
                    language_bar::render_bar(langs, inner.min(40), &colors, no_color);
                lines.extend(bar_lines);
            }

    // Streaks section
    if opts.show_streaks()
        && let Some(ref s) = profile.streaks {
            lines.push(String::new());
            let header = if no_color {
                "Streaks".to_string()
            } else {
                colors.title("Streaks")
            };
            lines.push(header);

            let streak_lines = streak::render_streaks(s, &colors, no_color);
            lines.extend(streak_lines);
        }

    // Repos table
    if opts.show_repos()
        && let Some(ref repos) = profile.top_repos
            && !repos.is_empty() {
                lines.push(String::new());
                let header = if no_color {
                    "Top Repositories".to_string()
                } else {
                    colors.title("Top Repositories")
                };
                lines.push(header);

                for r in repos {
                    let lang = r.language.as_deref().unwrap_or("");
                    let line = if no_color {
                        format!(
                            "  {} ★{} 🍴{}  {lang}",
                            r.name,
                            format_number(r.stars),
                            format_number(r.forks)
                        )
                    } else {
                        format!(
                            "  {} {}{}  {}",
                            colors.accent(&r.name),
                            colors.value(&format!("★{}", format_number(r.stars))),
                            colors.muted(&format!(" ⑂{}", format_number(r.forks))),
                            colors.muted(lang)
                        )
                    };
                    lines.push(line);
                }
            }

    // Render the card
    println!("{}", top_border(width, &colors, no_color));
    for line in &lines {
        println!("{}", card_line(line, width, &colors, no_color));
    }
    println!("{}", bottom_border(width, &colors, no_color));
}
