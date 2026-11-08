//! the app's palettes and type (apps/mobile/src/theme.ts), on geist.

use std::sync::LazyLock;

use rune::{Color, TextStyle, color::hex};

use crate::snapshot::Provider;

pub static FONTS: &[&[u8]] = &[
    include_bytes!("../../mobile/assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../../mobile/assets/fonts/Geist-Medium.ttf"),
    include_bytes!("../../mobile/assets/fonts/Geist-SemiBold.ttf"),
];
pub const FAMILIES: (&str, &str) = ("Geist", "Geist");

#[derive(Clone, Copy)]
pub struct Palette {
    /// window background.
    pub bg: Color,
    /// cards, one step above the background.
    pub surface: Color,
    /// toggles and empty bar tracks, one step above a card.
    pub raised: Color,
    pub fg: Color,
    /// secondary text.
    pub muted: Color,
    /// labels and hints.
    pub faint: Color,
    /// the activity highlight: a cli mid-turn.
    pub active: Color,
    /// the band sweeping across an active bar.
    pub sheen: Color,
    claude: Color,
    chatgpt: Color,
}

impl Palette {
    /// each cli's tone for its limit bars.
    pub fn tone(&self, provider: Provider) -> Color {
        match provider {
            Provider::Claude => self.claude,
            Provider::Chatgpt => self.chatgpt,
        }
    }
}

fn color(s: &str) -> Color {
    hex(s).unwrap_or(Color::BLACK)
}

pub static DARK: LazyLock<Palette> = LazyLock::new(|| Palette {
    bg: color("#0d0d0d"),
    surface: color("#18181a"),
    raised: color("#28282b"),
    fg: color("#f4f4f5"),
    muted: color("#a0a0a8"),
    faint: color("#6e6e76"),
    active: color("#4ade80"),
    sheen: color("#dcfce7"),
    claude: color("#d97757"),
    chatgpt: color("#e6e6e6"),
});

pub static LIGHT: LazyLock<Palette> = LazyLock::new(|| Palette {
    bg: color("#f2f2f3"),
    surface: color("#ffffff"),
    raised: color("#e4e4e7"),
    fg: color("#18181b"),
    muted: color("#63636b"),
    faint: color("#8e8e96"),
    active: color("#16a34a"),
    sheen: color("#86efac"),
    claude: color("#d97757"),
    chatgpt: color("#3c3c43"),
});

/// the notch, black like the screen edge it hangs from in either scheme.
pub const NOTCH: Color = Color::BLACK;
/// limit bars' tracks inside the notch.
pub static TRACK: LazyLock<Color> = LazyLock::new(|| color("#1a1a1a"));

/// the palette for the system's scheme.
pub fn palette(dark: bool) -> Palette {
    if dark { *DARK } else { *LIGHT }
}

/// a size with the app's -2% tracking.
const fn tracked(size: f32, weight: f32) -> TextStyle {
    TextStyle::new(size, weight).tracking(size * -0.02)
}

pub const TITLE: TextStyle = tracked(20.0, 600.0);
pub const HEADING: TextStyle = tracked(14.0, 600.0);
pub const BODY: TextStyle = tracked(13.0, 500.0);
pub const LABEL: TextStyle = tracked(12.0, 500.0);
/// digits share one width, so changing numbers don't shift their row.
pub const NUMERIC: TextStyle = tracked(12.0, 500.0).features("'tnum' 1");
