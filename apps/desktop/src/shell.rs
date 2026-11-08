//! what the tray, the settings window and the companion thread share: the windows' handles
//! and the prefs.

use std::sync::{Mutex, OnceLock, PoisonError};

use rune::{AppHandle, OverlayHandle, ScreenEdge, tray::Tray};

use crate::{companion::Link, format::now_ms, notch::Notch, prefs::Prefs, settings::Settings, snapshot::Snapshot, tray};

pub struct Shell {
    notch: OverlayHandle<Notch>,
    settings: AppHandle<Settings>,
    prefs: Mutex<Prefs>,
    tray: OnceLock<Tray>,
}

static SHELL: OnceLock<Shell> = OnceLock::new();

/// set once in `main`, before anything that calls `shell()` starts.
pub fn init(shell: Shell) {
    let _ = SHELL.set(shell);
}

#[allow(clippy::expect_used)]
pub fn shell() -> &'static Shell {
    SHELL.get().expect("main sets the shell before starting the app")
}

impl Shell {
    pub fn new(notch: OverlayHandle<Notch>, settings: AppHandle<Settings>, prefs: Prefs) -> Shell {
        Shell { notch, settings, prefs: Mutex::new(prefs), tray: OnceLock::new() }
    }

    pub fn prefs(&self) -> Prefs {
        *self.prefs.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// keeps the tray icon alive for as long as the app runs.
    pub fn set_tray(&self, tray: Tray) {
        let _ = self.tray.set(tray);
    }

    /// changes the prefs, saves them and hands the settings window the new ones.
    fn change(&self, f: impl FnOnce(&mut Prefs)) {
        let prefs = {
            let mut prefs = self.prefs.lock().unwrap_or_else(PoisonError::into_inner);
            f(&mut prefs);
            *prefs
        };
        if let Err(e) = prefs.save() {
            eprintln!("octo: couldn't save prefs: {e}");
        }
        self.settings.update(move |s, _| s.prefs = prefs);
    }

    pub fn set_show_notch(&self, show: bool) {
        self.change(|prefs| prefs.show_notch = show);
        if show { self.notch.show() } else { self.notch.hide() }
        if let Some(tray) = self.tray.get() {
            tray.set_menu(tray::menu(show));
        }
    }

    pub fn set_notch_offset(&self, offset: f32) {
        self.change(|prefs| prefs.notch_offset = offset);
        self.notch.place(ScreenEdge::Top, offset);
        self.notch.update(move |n, _| n.offset = offset);
    }

    pub fn set_snapshot(&self, snapshot: Snapshot) {
        self.notch.update(move |n, _| {
            n.snapshot = Some(snapshot);
            n.link = Link::Live;
            n.now = now_ms();
        });
        self.settings.update(|s, _| s.link = Link::Live);
    }

    pub fn set_link(&self, link: Link) {
        self.notch.update(move |n, _| n.link = link);
        self.settings.update(move |s, _| s.link = link);
    }

    /// moves the notch's countdowns on.
    pub fn tick(&self) {
        self.notch.update(|n, _| n.now = now_ms());
    }

    pub fn open_settings(&self) {
        self.settings.open();
    }

    pub fn quit(&self) {
        self.settings.quit();
    }
}
