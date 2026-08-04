use std::collections::BTreeMap;

use clap::ValueEnum;
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeKind {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn resolve(self) -> ThemeKind {
        match self {
            Self::Auto => detect_terminal_theme().unwrap_or(ThemeKind::Dark),
            Self::Dark => ThemeKind::Dark,
            Self::Light => ThemeKind::Light,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub foreground: Color,
    pub card_background: Color,
    pub card_border: Color,
    pub card_shadow: Color,
    pub muted: Color,
    pub accent: Color,
    pub comparison: Color,
    pub tab_active_fg: Color,
    pub tab_active_bg: Color,
    pub heat_0: Color,
    pub heat_3: Color,
    pub model_series: [Color; 12],
}

impl Theme {
    pub fn builtin_dark() -> Self {
        Self {
            foreground: Color::Rgb(229, 233, 240),
            card_background: Color::Rgb(26, 26, 26),
            card_border: Color::Rgb(120, 130, 155),
            card_shadow: Color::Rgb(0, 0, 0),
            muted: Color::Rgb(128, 134, 152),
            accent: Color::Rgb(136, 192, 208),
            comparison: Color::Rgb(180, 190, 254),
            tab_active_fg: Color::Black,
            tab_active_bg: Color::Rgb(136, 192, 208),
            heat_0: Color::Rgb(94, 98, 115),
            heat_3: Color::Rgb(136, 192, 208),
            model_series: [
                Color::Rgb(191, 97, 106),
                Color::Rgb(208, 135, 112),
                Color::Rgb(235, 203, 139),
                Color::Rgb(163, 190, 140),
                Color::Rgb(136, 192, 208),
                Color::Rgb(129, 161, 193),
                Color::Rgb(180, 142, 173),
                Color::Rgb(171, 121, 103),
                Color::Rgb(94, 129, 172),
                Color::Rgb(143, 188, 187),
                Color::Rgb(216, 222, 233),
                Color::Rgb(76, 86, 106),
            ],
        }
    }

    pub fn builtin_light() -> Self {
        Self {
            foreground: Color::Rgb(37, 41, 51),
            card_background: Color::Rgb(252, 253, 255),
            card_border: Color::Rgb(173, 183, 201),
            card_shadow: Color::Rgb(96, 107, 128),
            muted: Color::Rgb(90, 98, 115),
            accent: Color::Rgb(0, 122, 163),
            comparison: Color::Rgb(94, 92, 230),
            tab_active_fg: Color::White,
            tab_active_bg: Color::Rgb(0, 122, 163),
            heat_0: Color::Rgb(160, 170, 186),
            heat_3: Color::Rgb(0, 122, 163),
            model_series: [
                Color::Rgb(167, 40, 40),
                Color::Rgb(175, 94, 0),
                Color::Rgb(145, 108, 0),
                Color::Rgb(51, 122, 68),
                Color::Rgb(0, 122, 163),
                Color::Rgb(50, 88, 160),
                Color::Rgb(126, 76, 142),
                Color::Rgb(120, 78, 52),
                Color::Rgb(72, 106, 154),
                Color::Rgb(70, 150, 154),
                Color::Rgb(34, 34, 34),
                Color::Rgb(90, 98, 115),
            ],
        }
    }

    pub fn builtin_for(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Dark => Self::builtin_dark(),
            ThemeKind::Light => Self::builtin_light(),
        }
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn comparison_style(&self) -> Style {
        Style::default()
            .fg(self.comparison)
            .add_modifier(Modifier::BOLD)
    }

    pub fn series_color(&self, index: usize) -> Color {
        self.model_series[index % self.model_series.len()]
    }
}

#[derive(Clone, Debug)]
pub struct NamedTheme {
    pub kind: ThemeKind,
    pub theme: Theme,
}

pub fn builtin_themes() -> BTreeMap<String, NamedTheme> {
    let mut themes = BTreeMap::new();
    themes.insert(
        "dark".to_string(),
        NamedTheme {
            kind: ThemeKind::Dark,
            theme: Theme::builtin_for(ThemeKind::Dark),
        },
    );
    themes.insert(
        "light".to_string(),
        NamedTheme {
            kind: ThemeKind::Light,
            theme: Theme::builtin_for(ThemeKind::Light),
        },
    );
    themes
}

fn detect_terminal_theme() -> Option<ThemeKind> {
    let env_hint = std::env::var("TERM_THEME")
        .ok()
        .or_else(|| std::env::var("TERM_BACKGROUND").ok());
    if let Some(mode) = env_hint.and_then(|value| parse_mode_hint(&value)) {
        return Some(mode);
    }

    if let Some(mode) = detect_terminal_theme_from_luma() {
        return Some(mode);
    }

    std::env::var("COLORFGBG")
        .ok()
        .and_then(|value| parse_colorfgbg(&value))
}

fn detect_terminal_theme_from_luma() -> Option<ThemeKind> {
    terminal_light::luma().ok().map(|luma| {
        if luma > 0.6 {
            ThemeKind::Light
        } else {
            ThemeKind::Dark
        }
    })
}

fn parse_mode_hint(value: &str) -> Option<ThemeKind> {
    let lowercase = value.trim().to_ascii_lowercase();
    if lowercase.contains("dark") {
        return Some(ThemeKind::Dark);
    }
    if lowercase.contains("light") {
        return Some(ThemeKind::Light);
    }
    None
}

fn parse_colorfgbg(value: &str) -> Option<ThemeKind> {
    let background = value.split(';').next_back()?.trim().parse::<u8>().ok()?;
    if background <= 6 || background == 8 {
        Some(ThemeKind::Dark)
    } else {
        Some(ThemeKind::Light)
    }
}

#[cfg(test)]
mod tests {
    use super::{ThemeKind, parse_colorfgbg, parse_mode_hint};

    #[test]
    fn parses_term_theme_hint() {
        assert_eq!(parse_mode_hint("dark"), Some(ThemeKind::Dark));
        assert_eq!(parse_mode_hint("LIGHT"), Some(ThemeKind::Light));
        assert_eq!(parse_mode_hint("unknown"), None);
    }

    #[test]
    fn parses_colorfgbg_background_index() {
        assert_eq!(parse_colorfgbg("15;0"), Some(ThemeKind::Dark));
        assert_eq!(parse_colorfgbg("0;15"), Some(ThemeKind::Light));
    }
}
