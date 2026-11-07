//! the companion's `Snapshot` (packages/core/src/companion.ts), only the parts the desktop shows.

use serde::Deserialize;

/// the snapshot version this app understands.
const VERSION: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Claude,
    Chatgpt,
}

impl Provider {
    /// display order, as in the app.
    pub const ALL: [Provider; 2] = [Provider::Claude, Provider::Chatgpt];

    pub fn name(self) -> &'static str {
        match self {
            Provider::Claude => "Claude",
            Provider::Chatgpt => "ChatGPT",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Provider::Claude => "claude",
            Provider::Chatgpt => "chatgpt",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PerProvider<T> {
    pub claude: Option<T>,
    pub chatgpt: Option<T>,
}

impl<T> PerProvider<T> {
    pub fn get(&self, provider: Provider) -> Option<&T> {
        match provider {
            Provider::Claude => self.claude.as_ref(),
            Provider::Chatgpt => self.chatgpt.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Snapshot {
    pub usage: PerProvider<Usage>,
    /// whether each cli is mid-turn on the pc.
    #[serde(default)]
    pub working: PerProvider<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Usage {
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub kind: Kind,
    pub scope: Scope,
    pub used_percent: f64,
    /// epoch ms, null until the window starts.
    pub resets_at: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Session,
    Weekly,
    Monthly,
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    General,
    Model,
    Extra,
    #[serde(other)]
    Other,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// a companion speaking another snapshot version.
    Version(u32),
    Shape,
}

#[derive(Deserialize)]
struct Versioned {
    version: u32,
}

impl Snapshot {
    pub fn parse(raw: &str) -> Result<Snapshot, ParseError> {
        let versioned: Versioned = serde_json::from_str(raw).map_err(|_| ParseError::Shape)?;
        if versioned.version != VERSION {
            return Err(ParseError::Version(versioned.version));
        }
        serde_json::from_str(raw).map_err(|_| ParseError::Shape)
    }

    pub fn working(&self, provider: Provider) -> bool {
        self.working.get(provider).copied().unwrap_or(false)
    }
}

impl Usage {
    /// the plan's own window of `kind`, not a per-model or extra-usage one.
    pub fn window(&self, kind: Kind) -> Option<&UsageWindow> {
        self.windows.iter().find(|w| w.scope == Scope::General && w.kind == kind)
    }
}

impl UsageWindow {
    /// how much of the window is left, 0 to 100.
    pub fn left(&self) -> f64 {
        100.0 - self.used_percent.clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAPSHOT: &str = r#"{
        "version": 3, "sentAt": 1, "instanceId": "a", "sequence": 4,
        "usage": {
            "claude": {
                "provider": "claude", "plan": "max", "account": null, "source": "companion-api",
                "sourceId": "claude-code", "confirmedAt": 1, "fetchedAt": 1,
                "windows": [
                    { "id": "session", "label": "5h", "kind": "session", "scope": "general", "durationMs": 18000000, "usedPercent": 13, "resetsAt": 2 },
                    { "id": "opus", "label": "Opus · 7d", "kind": "weekly", "scope": "model", "durationMs": 604800000, "usedPercent": 90, "resetsAt": 3 },
                    { "id": "weekly", "label": "7d", "kind": "weekly", "scope": "general", "durationMs": 604800000, "usedPercent": 140, "resetsAt": null }
                ]
            }
        },
        "sources": { "claude": { "state": "ok" } },
        "working": { "claude": true }
    }"#;

    #[test]
    fn reads_the_plan_windows() {
        let snapshot = Snapshot::parse(SNAPSHOT).unwrap();
        let usage = snapshot.usage.get(Provider::Claude).unwrap();
        assert_eq!(usage.window(Kind::Session).unwrap().left(), 87.0);
        // the per-model weekly window isn't the plan's
        assert_eq!(usage.window(Kind::Weekly).unwrap().left(), 0.0);
        assert!(snapshot.usage.get(Provider::Chatgpt).is_none());
        assert!(snapshot.working(Provider::Claude));
        assert!(!snapshot.working(Provider::Chatgpt));
    }

    #[test]
    fn refuses_other_versions() {
        let older = SNAPSHOT.replacen("\"version\": 3", "\"version\": 2", 1);
        assert_eq!(Snapshot::parse(&older).unwrap_err(), ParseError::Version(2));
    }
}
