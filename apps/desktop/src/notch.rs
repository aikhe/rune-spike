//! the notch: a black pill hanging from the top of the screen with each cli's session left,
//! opening on hover into both windows with their resets. it stays dark in a light scheme,
//! like the screen edge it hangs from.

use rune::{
    Align, ConfiguredOverlay, Cx, Easing, El, LoopSpec, OverlayInput, OverlayOptions, OverlayPlacement, OverlaySpec,
    ScreenEdge, Spring, Transition, col, el, row, text,
};

use crate::{
    companion::Link,
    format::{duration, now_ms},
    snapshot::{Kind, Provider, Snapshot, Usage, UsageWindow},
    theme::{self, DARK, NOTCH, Palette, TRACK},
};

/// the overlay's fixed canvas, room for the opened notch; the window never resizes, so
/// only the pill's own shape takes clicks and hover.
const CANVAS: (f32, f32) = (400.0, 260.0);
const CLOSED: (f32, f32) = (200.0, 30.0);
const OPEN_WIDTH: f32 = 340.0;
const RADIUS: f32 = 16.0;
/// square where it meets the screen edge.
const HANGING: [f32; 4] = [0.0, 0.0, RADIUS, RADIUS];
/// lowered below a bar at the top of the screen, it rounds all round.
const FLOATING: [f32; 4] = [RADIUS; 4];
/// the app's sweep: a band 40% of the fill wide crosses it at a constant speed in the first
/// 60% of each 2s, from a band's width before it to just past its end (2.5 bands in), then
/// carries on out of sight for the rest.
const SWEEP: LoopSpec = LoopSpec::slide(2.0, [-1.0, 0.0], [-1.0 + 3.5 / 0.6, 0.0]).easing(Easing::LINEAR);

pub struct Notch {
    pub snapshot: Option<Snapshot>,
    pub link: Link,
    pub open: bool,
    pub now: f64,
    /// points below the top of the screen.
    pub offset: f32,
}

impl Notch {
    pub fn new(offset: f32) -> Self {
        Notch { snapshot: None, link: Link::Connecting, open: false, now: now_ms(), offset }
    }
}

pub fn overlay(offset: f32) -> ConfiguredOverlay<Notch> {
    OverlaySpec {
        name: "octo",
        size: CANVAS,
        edge: ScreenEdge::Top,
        margin: offset,
        align: Align::Center,
        inset: 0.0,
        interactive: false,
        view,
    }
    .options(OverlayOptions {
        input: OverlayInput::Regions,
        prewarm: true,
        placement: Some(placement(offset)),
        ..Default::default()
    })
}

/// `offset` points down from the top of the screen. an edge placement measures from the
/// display itself, where the spec's margin would start below app bars such as yasb's.
pub fn placement(offset: f32) -> OverlayPlacement {
    OverlayPlacement::Edge { edge: ScreenEdge::Top, margin: offset, align: Align::Center, inset: 0.0 }
}

fn view(n: &Notch, _: &mut Cx<Notch>) -> El {
    let p = *DARK;
    let corners = if n.offset > 0.0 { FLOATING } else { HANGING };
    let pill = col()
        .key("notch")
        .hit()
        .w(if n.open { OPEN_WIDTH } else { CLOSED.0 })
        .corners(corners)
        .clip_corners(corners)
        .fill(NOTCH)
        .motion(Spring::SNAPPY)
        .on_hover_change(|n: &mut Notch, inside, _| n.open = inside);
    let pill = if n.open { pill.padding(16.0).gap(16.0).child(opened(n, p)) } else { pill.h(CLOSED.1).child(closed(n, p)) };
    // the root stays clear, so everything around the pill is transparent
    col().w_full().h_full().items_center().child(pill)
}

/// each provider with usage, in display order.
fn providers(n: &Notch) -> Vec<(Provider, &Usage, bool)> {
    let Some(snapshot) = &n.snapshot else { return Vec::new() };
    Provider::ALL
        .into_iter()
        .filter_map(|provider| Some((provider, snapshot.usage.get(provider)?, snapshot.working(provider))))
        .collect()
}

/// what's left of a window in the cli's tone, or in the activity highlight while the cli works.
/// `sweep` also runs the band across it then; the closed pill stays still, so a long turn
/// doesn't redraw the screen the whole time.
fn bar(key: String, left: f64, provider: Provider, working: bool, sweep: bool, p: Palette, height: f32) -> El {
    let radius = height / 2.0;
    let fill = el()
        .key(format!("{key}-fill"))
        .w_frac(left as f32 / 100.0)
        .h_full()
        .radius(radius)
        .fill(if working { p.active } else { p.tone(provider) })
        .motion(Spring::SNAPPY);
    let fill = if working && sweep {
        fill.clip_rounded(radius)
            .child(el().key(format!("{key}-sweep")).w_frac(0.4).h_full().fill(p.sheen.alpha(0.35)).looping(SWEEP))
    } else {
        fill
    };
    el().key(key).h(height).radius(radius).fill(*TRACK).child(fill)
}

/// a session gauge on each side, like the notch's ears.
fn closed(n: &Notch, p: Palette) -> El {
    let shown = providers(n);
    let glance = row().key("glance").w_full().h_full().px(14.0).items_center().transition(Transition::opacity());
    if shown.is_empty() {
        let label = if n.link == Link::Live { "octo" } else { n.link.message() };
        return glance.center().child(text(label).font(theme::LABEL).color(p.faint).truncate());
    }
    let glance = if shown.len() == 1 { glance.center() } else { glance.justify_between() };
    shown.into_iter().fold(glance, |glance, (provider, usage, working)| {
        let Some(session) = usage.window(Kind::Session) else { return glance };
        let left = session.left();
        glance.child(
            row()
                .key(format!("{}-glance", provider.key()))
                .gap(6.0)
                .items_center()
                .child(bar(format!("{}-mini", provider.key()), left, provider, working, false, p, 4.0).w(36.0))
                .child(text(format!("{}%", left.round())).font(theme::NUMERIC).color(p.fg)),
        )
    })
}

fn opened(n: &Notch, p: Palette) -> El {
    let shown = providers(n);
    let mut body = col().key("opened").w_full().gap(16.0).transition(Transition::opacity());
    for (provider, usage, working) in &shown {
        body = body.child(section(*provider, usage, *working, n.now, p));
    }
    if shown.is_empty() || n.link != Link::Live {
        body = body.child(text(n.link.message()).key("link").font(theme::LABEL).color(p.faint));
    }
    body
}

fn section(provider: Provider, usage: &Usage, working: bool, now: f64, p: Palette) -> El {
    let status = if working { "Working" } else { usage.plan.as_deref().unwrap_or_default() };
    let header = row()
        .key(format!("{}-header", provider.key()))
        .w_full()
        .items_center()
        .justify_between()
        .child(text(provider.name()).font(theme::HEADING).color(p.fg))
        .child(text(capitalized(status)).font(theme::LABEL).color(if working { p.active } else { p.faint }));
    [(Kind::Session, "Session"), (Kind::Weekly, "Weekly")].into_iter().fold(
        col().key(format!("{}-section", provider.key())).w_full().gap(8.0).child(header),
        |section, (kind, name)| match usage.window(kind) {
            Some(window) => section.child(window_row(provider, name, window, working, now, p)),
            None => section,
        },
    )
}

/// a window's name, a bar of what's left, the share and the time until it resets.
fn window_row(provider: Provider, name: &'static str, w: &UsageWindow, working: bool, now: f64, p: Palette) -> El {
    let key = format!("{}-{}", provider.key(), name.to_lowercase());
    let left = w.left();
    let reset = w.resets_at.map(|at| duration(at - now)).unwrap_or_default();
    row()
        .key(key.clone())
        .w_full()
        .items_center()
        .gap(10.0)
        .child(row().w(56.0).child(text(name).font(theme::LABEL).color(p.muted)))
        .child(bar(format!("{key}-bar"), left, provider, working, true, p, 6.0).grow(1.0))
        .child(row().w(36.0).justify_end().child(text(format!("{}%", left.round())).font(theme::NUMERIC).color(p.fg)))
        .child(row().w(52.0).justify_end().child(text(reset).font(theme::NUMERIC).color(p.faint)))
}

/// plans arrive lowercase ("max", "plus").
fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}
