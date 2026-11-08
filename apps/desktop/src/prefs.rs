use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

/// where the companion keeps its token, beside which the desktop keeps its prefs.
pub fn octo_home() -> PathBuf {
    std::env::home_dir().unwrap_or_default().join(".octo")
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Prefs {
    pub show_notch: bool,
    /// points the notch sits below the top of the screen, to clear a bar there.
    pub notch_offset: f32,
    /// the companion's port on this pc.
    pub port: u16,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { show_notch: true, notch_offset: 0.0, port: 8787 }
    }
}

impl Prefs {
    fn path() -> PathBuf {
        octo_home().join("desktop.json")
    }

    /// the saved prefs, defaults when there are none or they don't read.
    pub fn load() -> Prefs {
        fs::read_to_string(Self::path())
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        fs::create_dir_all(octo_home())?;
        // rename so a crash never leaves half a file
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(tmp, path)
    }
}

/// starting octo with windows, through the current user's run key.
pub mod autostart {
    #[cfg(windows)]
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};

    #[cfg(windows)]
    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    #[cfg(windows)]
    const NAME: &str = "octo";

    #[cfg(windows)]
    pub fn enabled() -> bool {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(RUN)
            .and_then(|run| run.get_value::<String, _>(NAME))
            .is_ok()
    }

    #[cfg(windows)]
    pub fn set(on: bool) -> std::io::Result<()> {
        let (run, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(RUN)?;
        if on {
            let exe = std::env::current_exe()?;
            run.set_value(NAME, &format!("\"{}\"", exe.display()))
        } else {
            run.delete_value(NAME)
        }
    }

    #[cfg(not(windows))]
    pub fn enabled() -> bool {
        false
    }

    #[cfg(not(windows))]
    pub fn set(_: bool) -> std::io::Result<()> {
        Err(std::io::ErrorKind::Unsupported.into())
    }
}
