//! the app's dark palette and type (apps/mobile/src/theme.ts), on geist.

use std::sync::LazyLock;

use rune::{Color, TextStyle, hex};

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
    /// the notch, black like the screen edge it hangs from.
    pub notch: Color,
    /// limit bars' tracks inside the notch.
    pub track: Color,
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
    notch: color("#000000"),
    track: color("#1a1a1a"),
    claude: color("#d97757"),
    chatgpt: color("#e6e6e6"),
});

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
