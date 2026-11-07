//! streams the companion's snapshots from `/live` on this pc.

use std::{fs, thread, time::Duration};

use serde::Deserialize;
use tungstenite::{
    Message,
    client::IntoClientRequest,
    http::{HeaderValue, header::AUTHORIZATION},
    stream::MaybeTlsStream,
};

use crate::{
    prefs::octo_home,
    shell::shell,
    snapshot::{ParseError, Snapshot},
};

const FIRST_RETRY: Duration = Duration::from_secs(2);
const LAST_RETRY: Duration = Duration::from_secs(30);
// the companion pings every minute, so this long without a frame means the socket died
const READ_TIMEOUT: Duration = Duration::from_secs(90);

/// how the desktop's link to the companion is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Connecting,
    Live,
    /// nothing answers on the port.
    NotRunning,
    /// the companion has never run here, so there's no token yet.
    NoToken,
    /// the companion speaks another snapshot version.
    Outdated,
}

impl Link {
    pub fn message(self) -> &'static str {
        match self {
            Link::Connecting => "Connecting…",
            Link::Live => "Connected",
            Link::NotRunning => "Companion isn't running",
            Link::NoToken => "Run the companion once to set it up",
            Link::Outdated => "Update the companion to match this app",
        }
    }
}

#[derive(Deserialize)]
struct Config {
    token: String,
}

/// the token the companion issued, re-read each time since `rotate` replaces it.
fn token() -> Option<String> {
    let raw = fs::read_to_string(octo_home().join("companion.json")).ok()?;
    serde_json::from_str::<Config>(&raw).ok().map(|config| config.token)
}

/// connects, and reconnects with backoff for as long as the app runs.
pub fn start(port: u16) {
    thread::spawn(move || {
        let mut retry = FIRST_RETRY;
        loop {
            let (link, streamed) = stream(port);
            shell().set_link(link);
            retry = if streamed { FIRST_RETRY } else { (retry * 2).min(LAST_RETRY) };
            thread::sleep(retry);
        }
    });
}

/// reads snapshots until the socket ends; returns why, and whether any arrived.
fn stream(port: u16) -> (Link, bool) {
    let Some(token) = token() else { return (Link::NoToken, false) };
    let Ok(mut request) = format!("ws://127.0.0.1:{port}/live").into_client_request() else {
        return (Link::NotRunning, false);
    };
    let Ok(auth) = HeaderValue::from_str(&format!("Bearer {token}")) else {
        return (Link::NoToken, false);
    };
    request.headers_mut().insert(AUTHORIZATION, auth);
    let Ok((mut socket, _)) = tungstenite::connect(request) else {
        return (Link::NotRunning, false);
    };
    if let MaybeTlsStream::Plain(tcp) = socket.get_ref() {
        // without it a vanished companion (the pc slept) would hold the socket open forever
        let _ = tcp.set_read_timeout(Some(READ_TIMEOUT));
    }
    let mut streamed = false;
    loop {
        let Ok(message) = socket.read() else { return (Link::NotRunning, streamed) };
        let Message::Text(raw) = message else { continue };
        match Snapshot::parse(raw.as_str()) {
            Ok(snapshot) => {
                streamed = true;
                shell().set_snapshot(snapshot);
            }
            Err(ParseError::Version(_)) => return (Link::Outdated, streamed),
            // one malformed snapshot shouldn't drop the ones after it
            Err(ParseError::Shape) => {}
        }
    }
}
