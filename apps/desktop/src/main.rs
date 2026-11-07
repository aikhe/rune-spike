// a tray app: no console window behind it
#![windows_subsystem = "windows"]

mod companion;
mod format;
mod notch;
mod prefs;
mod settings;
mod shell;
mod snapshot;
mod theme;
mod tray;

use std::{thread, time::Duration};

use rune::{App, AppSpec, WindowOptions};

use crate::{
    prefs::Prefs,
    shell::{Shell, shell},
};

// countdowns read in minutes, so this keeps them current
const TICK: Duration = Duration::from_secs(30);

fn main() {
    let prefs = Prefs::load();
    // the settings window is the app's main window; it starts closed and closing it keeps octo running
    let mut app = App::new(AppSpec {
        fonts: theme::FONTS,
        families: theme::FAMILIES,
        min_size: Some(settings::SIZE),
        ..AppSpec::new("octo", settings::SIZE, settings::init, settings::view)
    })
    .windowless()
    .keep_running()
    .window_options(WindowOptions { resizable: false, ..Default::default() });
    let notch = app.overlay(notch::overlay(), notch::Notch::default());
    if prefs.show_notch {
        notch.show();
    }
    shell::init(Shell::new(notch, app.handle(), prefs));
    tray::start();
    companion::start(prefs.port);
    thread::spawn(|| {
        loop {
            thread::sleep(TICK);
            shell().tick();
        }
    });
    app.run();
}
