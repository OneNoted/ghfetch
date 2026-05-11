use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ghfetch",
    about = "GitHub stats in the terminal, neofetch-style"
)]
#[command(version, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Username (shorthand for `ghfetch user <username>`)
    #[arg(value_name = "USERNAME")]
    pub username: Option<String>,

    /// GitHub personal access token
    #[arg(long, global = true, env = "GITHUB_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Output raw JSON instead of a card
    #[arg(long, global = true)]
    pub json: bool,

    /// Color theme
    #[arg(long, global = true, value_enum, default_value_t = Theme::Mocha)]
    pub theme: Theme,

    /// Show verbose output (rate limits, API calls)
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Display user profile stats
    User {
        /// GitHub username
        username: String,

        #[command(flatten)]
        opts: UserOpts,
    },

    /// Display repository stats
    Repo {
        /// Repository in owner/repo, GitHub URL, or GitHub SSH format
        repo: String,

        #[command(flatten)]
        opts: RepoOpts,
    },

    /// Display organization stats
    Org {
        /// Organization name
        orgname: String,

        #[command(flatten)]
        opts: OrgOpts,
    },

    /// Explain which repositories contribute to language totals
    Breakdown {
        /// GitHub username
        username: String,

        #[command(flatten)]
        opts: BreakdownOpts,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Theme {
    Mocha,
    Macchiato,
    Frappe,
    Latte,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum SortBy {
    Stars,
    Forks,
    Updated,
    Size,
    Name,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum BreakdownBy {
    Language,
    Repo,
}

#[derive(Parser)]
pub struct UserOpts {
    /// Show only the languages section
    #[arg(long)]
    pub languages: bool,

    /// Show only the streaks section
    #[arg(long)]
    pub streaks: bool,

    /// Show only the repos section
    #[arg(long)]
    pub repos: bool,

    /// Show only the contributions section
    #[arg(long)]
    pub contributions: bool,

    /// Show all sections (equivalent to --languages --streaks --repos --contributions)
    #[arg(long)]
    pub all: bool,

    /// Maximum number of languages to display (default: 10, or all with --languages)
    #[arg(long)]
    pub lang_limit: Option<usize>,

    /// Maximum number of repos to display
    #[arg(long, default_value_t = 10)]
    pub repo_limit: usize,

    /// Sort repos by this field
    #[arg(long, value_enum, default_value_t = SortBy::Stars)]
    pub sort_by: SortBy,

    /// Exclude forked repositories
    #[arg(long, visible_alias = "ignore-forks")]
    pub no_forks: bool,
}

#[derive(Parser)]
pub struct RepoOpts {
    /// Show only the languages section
    #[arg(long)]
    pub languages: bool,

    /// Show all sections
    #[arg(long)]
    pub all: bool,
}

#[derive(Parser)]
pub struct OrgOpts {
    /// Show only the languages section
    #[arg(long)]
    pub languages: bool,

    /// Show only the repos section
    #[arg(long)]
    pub repos: bool,

    /// Show all sections
    #[arg(long)]
    pub all: bool,

    /// Maximum number of languages to display (default: 10, or all with --languages)
    #[arg(long)]
    pub lang_limit: Option<usize>,

    /// Maximum number of repos to display
    #[arg(long, default_value_t = 10)]
    pub repo_limit: usize,

    /// Exclude forked repositories
    #[arg(long, visible_alias = "ignore-forks")]
    pub no_forks: bool,
}

#[derive(Parser)]
pub struct BreakdownOpts {
    /// Group breakdown by language or repository
    #[arg(long, value_enum, default_value_t = BreakdownBy::Language)]
    pub by: BreakdownBy,

    /// Show only this language
    #[arg(long)]
    pub language: Option<String>,

    /// Show only this repository (name or owner/name)
    #[arg(long)]
    pub repo: Option<String>,

    /// Maximum number of groups to display (0 means all)
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Maximum number of nested entries per group (0 means all)
    #[arg(long, default_value_t = 10)]
    pub repo_limit: usize,

    /// Exclude forked repositories
    #[arg(long, visible_alias = "ignore-forks")]
    pub no_forks: bool,
}

impl UserOpts {
    /// Create UserOpts for the shorthand `ghfetch <username>` form (show full card).
    pub fn default_full() -> Self {
        Self {
            languages: false,
            streaks: false,
            repos: false,
            contributions: false,
            all: false,
            lang_limit: None,
            repo_limit: 10,
            sort_by: SortBy::Stars,
            no_forks: false,
        }
    }

    /// Returns true if no section-specific flags were set (show full card).
    pub fn show_full_card(&self) -> bool {
        !self.languages && !self.streaks && !self.repos && !self.contributions && !self.all
    }

    pub fn show_languages(&self) -> bool {
        self.all || self.languages || self.show_full_card()
    }

    /// True when --languages was explicitly passed (detailed table view).
    pub fn detailed_languages(&self) -> bool {
        self.languages && !self.all && !self.show_full_card()
    }

    /// Effective lang limit: 0 (all) for detailed view unless explicitly set.
    pub fn effective_lang_limit(&self) -> usize {
        match self.lang_limit {
            Some(n) => n,
            None if self.detailed_languages() => 0,
            None => 10,
        }
    }

    pub fn show_streaks(&self) -> bool {
        self.all || self.streaks || self.show_full_card()
    }

    pub fn show_repos(&self) -> bool {
        self.all || self.repos
    }

    pub fn show_contributions(&self) -> bool {
        self.all || self.contributions || self.show_full_card()
    }
}

impl RepoOpts {
    pub fn show_full_card(&self) -> bool {
        !self.languages && !self.all
    }

    pub fn show_languages(&self) -> bool {
        self.all || self.languages || self.show_full_card()
    }

    pub fn detailed_languages(&self) -> bool {
        self.languages && !self.all && !self.show_full_card()
    }
}

impl OrgOpts {
    pub fn show_full_card(&self) -> bool {
        !self.languages && !self.repos && !self.all
    }

    pub fn show_languages(&self) -> bool {
        self.all || self.languages || self.show_full_card()
    }

    pub fn detailed_languages(&self) -> bool {
        self.languages && !self.all && !self.show_full_card()
    }

    pub fn effective_lang_limit(&self) -> usize {
        match self.lang_limit {
            Some(n) => n,
            None if self.detailed_languages() => 0,
            None => 10,
        }
    }

    pub fn show_repos(&self) -> bool {
        self.all || self.repos
    }
}
