use crate::cli::{BreakdownBy, BreakdownOpts, Theme};
use crate::data::breakdown::BreakdownProfile;
use crate::data::languages;
use crate::display::theme::ThemeColors;
use owo_colors::OwoColorize;

pub fn render(profile: &BreakdownProfile, opts: &BreakdownOpts, theme: Theme, no_color: bool) {
    let colors = ThemeColors::from_theme(theme);

    if profile.total_bytes == 0 {
        println!("No language data available for this breakdown.");
        return;
    }

    let title = format!("{} repository language breakdown", profile.login);
    if no_color {
        println!("\n{title}");
    } else {
        println!("\n{}", colors.title(&title));
    }

    let subtitle = format!(
        "{} repos, {}, {} estimated LoC",
        profile.repo_count,
        languages::format_bytes(profile.total_bytes),
        languages::format_lines(profile.total_bytes)
    );
    if no_color {
        println!("{subtitle}");
    } else {
        println!("{}", colors.muted(&subtitle));
    }

    let note = "LoC is estimated from GitHub Linguist byte counts for owned repositories.";
    if no_color {
        println!("{note}");
    } else {
        println!("{}", colors.muted(note));
    }
    println!();

    match opts.by {
        BreakdownBy::Language => render_by_language(profile, &colors, no_color),
        BreakdownBy::Repo => render_by_repo(profile, &colors, no_color),
    }
}

fn render_by_language(profile: &BreakdownProfile, colors: &ThemeColors, no_color: bool) {
    for group in &profile.language_groups {
        let header = format!(
            "{}  {}  {}  {:.1}%  {} repos",
            group.name,
            languages::format_bytes(group.bytes),
            languages::format_lines(group.bytes),
            group.percentage,
            group.repo_count
        );
        print_header(&header, colors, no_color);

        for repo in &group.repos {
            let flags = repo_flags(repo.is_fork, repo.is_private);
            let line = format!(
                "  {:<36} {:>10} {:>12} {:>6.1}%  stars {:>5} forks {:>5}{}",
                truncate(&repo.name, 36),
                languages::format_bytes(repo.bytes),
                languages::format_lines(repo.bytes),
                repo.percentage_of_language,
                repo.stars,
                repo.forks,
                flags
            );
            print_line(&line, colors, no_color);
        }
        println!();
    }
}

fn render_by_repo(profile: &BreakdownProfile, colors: &ThemeColors, no_color: bool) {
    for repo in &profile.repo_groups {
        let flags = repo_flags(repo.is_fork, repo.is_private);
        let header = format!(
            "{}  {}  {}  {:.1}%  stars {} forks {}{}",
            repo.name,
            languages::format_bytes(repo.bytes),
            languages::format_lines(repo.bytes),
            repo.percentage,
            repo.stars,
            repo.forks,
            flags
        );
        print_header(&header, colors, no_color);

        for language in &repo.languages {
            let line = format!(
                "  {:<24} {:>10} {:>12} {:>6.1}%",
                truncate(&language.name, 24),
                languages::format_bytes(language.bytes),
                languages::format_lines(language.bytes),
                language.percentage_of_repo
            );
            print_line(&line, colors, no_color);
        }
        println!();
    }
}

fn print_header(header: &str, colors: &ThemeColors, no_color: bool) {
    if no_color {
        println!("{header}");
    } else {
        println!("{}", colors.accent(header).bold());
    }
}

fn print_line(line: &str, colors: &ThemeColors, no_color: bool) {
    if no_color {
        println!("{line}");
    } else {
        println!("{}", colors.value(line));
    }
}

fn repo_flags(is_fork: bool, is_private: bool) -> String {
    match (is_fork, is_private) {
        (true, true) => " [fork, private]".to_string(),
        (true, false) => " [fork]".to_string(),
        (false, true) => " [private]".to_string(),
        (false, false) => String::new(),
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars.saturating_sub(3)).collect();
    truncated.push_str("...");
    truncated
}
