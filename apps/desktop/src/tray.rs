use rune::tray::{Tray, TrayEvent, TrayIcon, TrayItem, TraySpec};

use crate::shell::shell;

const ICON: u32 = 32;

/// the tray's menu, naming what the notch item will do.
pub fn menu(show_notch: bool) -> Vec<TrayItem> {
    vec![
        TrayItem::action("settings", "Settings…"),
        TrayItem::action("notch", if show_notch { "Hide notch" } else { "Show notch" }),
        TrayItem::Separator,
        TrayItem::action("quit", "Quit octo"),
    ]
}

pub fn start() {
    let spec = TraySpec::new("octo", "octo", icon()).tooltip("octo").menu(menu(shell().prefs().show_notch));
    match Tray::new(spec, on_event) {
        Ok(tray) => shell().set_tray(tray),
        Err(e) => eprintln!("octo: no tray icon: {e}"),
    }
}

/// runs on the tray's own thread; every call it makes wakes the app.
fn on_event(event: TrayEvent) {
    match event {
        TrayEvent::Activate { .. } => shell().open_settings(),
        TrayEvent::Menu(id) => match id.as_str() {
            "settings" => shell().open_settings(),
            "notch" => shell().set_show_notch(!shell().prefs().show_notch),
            "quit" => shell().quit(),
            _ => {}
        },
        _ => {}
    }
}

/// a white disc with an antialiased edge, readable on light and dark taskbars' trays.
fn icon() -> TrayIcon {
    let center = ICON as f32 / 2.0;
    let radius = center - 4.0;
    let pixels = (0..ICON * ICON)
        .flat_map(|i| {
            let (x, y) = ((i % ICON) as f32 + 0.5, (i / ICON) as f32 + 0.5);
            let distance = ((x - center).powi(2) + (y - center).powi(2)).sqrt();
            let alpha = (radius + 0.5 - distance).clamp(0.0, 1.0);
            [255, 255, 255, (alpha * 255.0).round() as u8]
        })
        .collect();
    TrayIcon::rgba(ICON, ICON, pixels)
}
