//! the settings window: a sidebar of pages, each a card of rows.

use rune::{Color, Cx, El, Spring, col, el, row, text};

use crate::{
    companion::Link,
    prefs::{Prefs, autostart},
    shell::shell,
    theme::{self, Palette},
};

pub const SIZE: (f32, f32) = (680.0, 440.0);
/// how far the notch can be lowered, and by how much each press moves it, in points.
const OFFSET_MAX: f32 = 96.0;
const OFFSET_STEP: f32 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    General,
    Companion,
    About,
}

impl Page {
    const ALL: [Page; 3] = [Page::General, Page::Companion, Page::About];

    fn name(self) -> &'static str {
        match self {
            Page::General => "General",
            Page::Companion => "Companion",
            Page::About => "About",
        }
    }
}

pub struct Settings {
    pub page: Page,
    pub prefs: Prefs,
    pub autostart: bool,
    pub link: Link,
}

pub fn init() -> Settings {
    Settings { page: Page::General, prefs: Prefs::load(), autostart: autostart::enabled(), link: Link::Connecting }
}

pub fn view(s: &Settings, cx: &mut Cx<Settings>) -> El {
    let p = theme::palette(cx.system_dark());
    let sidebar = Page::ALL.into_iter().fold(col().key("sidebar").w(180.0).h_full().padding(12.0).gap(2.0), |nav, page| {
        nav.child(nav_item(page, s.page == page, p))
    });
    let body = match s.page {
        Page::General => general(s, p),
        Page::Companion => companion(s, p),
        Page::About => about(p),
    };
    row().w_full().h_full().fill(p.bg).child(sidebar).child(
        col()
            .key("page")
            .grow(1.0)
            .h_full()
            .padding(24.0)
            .gap(16.0)
            .child(text(s.page.name()).key("title").font(theme::TITLE).color(p.fg))
            .child(body),
    )
}

fn nav_item(page: Page, selected: bool, p: Palette) -> El {
    row()
        .key(format!("nav-{}", page.name()))
        .role("tab")
        .label(page.name())
        .checked(selected)
        .h(32.0)
        .px(12.0)
        .items_center()
        .radius(8.0)
        .fill(if selected { p.surface } else { Color::TRANSPARENT })
        .hover(p.surface)
        .on_press(move |s: &mut Settings| s.page = page)
        .child(text(page.name()).font(theme::BODY).color(if selected { p.fg } else { p.muted }))
}

fn card(key: &str, p: Palette) -> El {
    col().key(key.to_owned()).w_full().radius(12.0).fill(p.surface).py(4.0)
}

/// a titled row with an optional hint below and a control on the right.
fn setting(key: &str, title: &str, hint: Option<&str>, control: El, p: Palette) -> El {
    let mut copy = col().gap(2.0).shrink().child(text(title.to_owned()).font(theme::BODY).color(p.fg));
    if let Some(hint) = hint {
        copy = copy.child(text(hint.to_owned()).font(theme::LABEL).color(p.faint).wrap());
    }
    row()
        .key(key.to_owned())
        .w_full()
        .px(16.0)
        .py(12.0)
        .gap(16.0)
        .items_center()
        .justify_between()
        .child(copy)
        .child(control)
}

/// a monochrome switch, as the app's: the knob slides, so on and off read as one motion.
fn toggle(key: &str, label: &str, on: bool, p: Palette, flip: fn(&mut Settings)) -> El {
    el().key(key.to_owned())
        .role("switch")
        .label(label.to_owned())
        .checked(on)
        .focusable(true)
        .size(38.0, 22.0)
        .radius(11.0)
        .fill(if on { p.fg } else { p.raised })
        .motion(Spring::SNAPPY)
        .on_press(flip)
        .child(
            el().key(format!("{key}-knob"))
                .size(16.0, 16.0)
                .radius(8.0)
                .fill(if on { p.bg } else { p.muted })
                .absolute(if on { 19.0 } else { 3.0 }, 3.0),
        )
}

/// a round − or + that moves the notch's offset by `delta`, dimmed at the end it can't pass.
fn nudge(key: &str, glyph: &'static str, label: &str, delta: f32, s: &Settings, p: Palette) -> El {
    let next = (s.prefs.notch_offset + delta).clamp(0.0, OFFSET_MAX);
    row()
        .key(key.to_owned())
        .role("button")
        .label(label.to_owned())
        .focusable(true)
        .size(26.0, 26.0)
        .radius(13.0)
        .center()
        .fill(p.raised)
        .on_press(move |s: &mut Settings| {
            s.prefs.notch_offset = next;
            shell().set_notch_offset(next);
        })
        .child(text(glyph).font(theme::BODY).color(if next == s.prefs.notch_offset { p.faint } else { p.fg }))
}

fn value(label: String, color: Color) -> El {
    text(label).font(theme::BODY).color(color)
}

fn general(s: &Settings, p: Palette) -> El {
    card("general", p)
        .child(setting(
            "autostart",
            "Start with Windows",
            None,
            toggle("autostart-toggle", "Start with Windows", s.autostart, p, |s| {
                if let Err(e) = autostart::set(!s.autostart) {
                    eprintln!("octo: couldn't change autostart: {e}");
                }
                s.autostart = autostart::enabled();
            }),
            p,
        ))
        .child(setting(
            "notch",
            "Show notch",
            Some("Hover the top of the screen to see your limits."),
            toggle("notch-toggle", "Show notch", s.prefs.show_notch, p, |s| {
                s.prefs.show_notch = !s.prefs.show_notch;
                shell().set_show_notch(s.prefs.show_notch);
            }),
            p,
        ))
        .child(setting(
            "offset",
            "Notch offset",
            Some("Lowers the notch below a bar at the top of your screen."),
            row()
                .gap(8.0)
                .items_center()
                .child(nudge("offset-down", "−", "Raise notch", -OFFSET_STEP, s, p))
                .child(row().w(28.0).justify_center().child(text(format!("{}", s.prefs.notch_offset)).font(theme::NUMERIC).color(p.fg)))
                .child(nudge("offset-up", "+", "Lower notch", OFFSET_STEP, s, p)),
            p,
        ))
}

fn companion(s: &Settings, p: Palette) -> El {
    let live = s.link == Link::Live;
    let status = row()
        .gap(8.0)
        .items_center()
        .child(el().size(8.0, 8.0).radius(4.0).fill(if live { p.active } else { p.faint }))
        .child(value(s.link.message().to_owned(), p.fg));
    card("companion", p)
        .child(setting("status", "Status", Some("Start it with `bun run companion` in the octo repo."), status, p))
        .child(setting("address", "Address", None, value(format!("127.0.0.1:{}", s.prefs.port), p.muted), p))
}

fn about(p: Palette) -> El {
    card("about", p).child(setting(
        "version",
        "octo for Windows",
        Some("Your Claude and ChatGPT limits, from the companion on this PC."),
        value(env!("CARGO_PKG_VERSION").to_owned(), p.muted),
        p,
    ))
}
