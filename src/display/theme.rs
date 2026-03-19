use crate::cli::Theme;
use catppuccin::Rgb;
use owo_colors::OwoColorize;

pub struct ThemeColors {
    pub title: Rgb,
    pub label: Rgb,
    pub value: Rgb,
    pub accent: Rgb,
    pub muted_color: Rgb,
    pub border_color: Rgb,
}

impl ThemeColors {
    pub fn from_theme(theme: Theme) -> Self {
        let flavor = match theme {
            Theme::Mocha => catppuccin::PALETTE.mocha,
            Theme::Macchiato => catppuccin::PALETTE.macchiato,
            Theme::Frappe => catppuccin::PALETTE.frappe,
            Theme::Latte => catppuccin::PALETTE.latte,
        };

        let colors = &flavor.colors;

        Self {
            title: colors.mauve.rgb,
            label: colors.blue.rgb,
            value: colors.text.rgb,
            accent: colors.green.rgb,
            muted_color: colors.overlay0.rgb,
            border_color: colors.surface1.rgb,
        }
    }

    pub fn title(&self, s: &str) -> String {
        format!("{}", s.truecolor(self.title.r, self.title.g, self.title.b).bold())
    }

    pub fn label(&self, s: &str) -> String {
        format!("{}", s.truecolor(self.label.r, self.label.g, self.label.b))
    }

    pub fn value(&self, s: &str) -> String {
        format!("{}", s.truecolor(self.value.r, self.value.g, self.value.b))
    }

    pub fn accent(&self, s: &str) -> String {
        format!("{}", s.truecolor(self.accent.r, self.accent.g, self.accent.b))
    }

    pub fn muted(&self, s: &str) -> String {
        format!("{}", s.truecolor(self.muted_color.r, self.muted_color.g, self.muted_color.b))
    }

    pub fn border(&self, s: &str) -> String {
        format!("{}", s.truecolor(self.border_color.r, self.border_color.g, self.border_color.b))
    }
}
