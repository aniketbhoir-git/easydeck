//! Runs the deck's data sources and pushes each reading to the UI as the Tauri event
//! `source:<id>`. The payload is the `SourceValue` object: metric name to number or `null`,
//! for example `{"cpu.load":12.5,"gpu.load":null}`.

use easydeck_core::deck::{Deck, Source as DeckSource};
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};

/// Holds the running source tasks. Managed by Tauri; dropping it stops them all.
#[derive(Default)]
pub struct SourceRunner {
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl SourceRunner {
    /// Stops the running sources, then starts every source of `deck` that `allowed` accepts.
    /// `allowed` is the approval gate: a source with an unapproved endpoint does not start
    /// (SPEC §4.3). Call it again when a new deck is loaded or an approval changes.
    pub fn restart(&self, app: &AppHandle, deck: &Deck, allowed: impl Fn(&DeckSource) -> bool) {
        self.stop();
        let started = start_all(app, deck, &allowed);
        *self.tasks.lock().unwrap_or_else(|e| e.into_inner()) = started;
    }

    pub fn stop(&self) {
        for t in self
            .tasks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .drain(..)
        {
            t.abort();
        }
    }
}

impl Drop for SourceRunner {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
fn start_all(
    app: &AppHandle,
    deck: &Deck,
    allowed: &dyn Fn(&DeckSource) -> bool,
) -> Vec<JoinHandle<()>> {
    use easydeck_core::sources::system_stats::SystemStatsSource;
    use easydeck_core::sources::Source;
    use easydeck_platform_windows::stats::WindowsStatsReader;

    let mut tasks = Vec::new();
    for (id, source) in &deck.sources {
        if !allowed(source) {
            continue;
        }
        if let DeckSource::SystemStats {
            interval_ms,
            metrics,
        } = source
        {
            let event = format!("source:{id}");
            let app = app.clone();
            let (interval_ms, metrics) = (*interval_ms, metrics.clone());
            tasks.push(tauri::async_runtime::spawn(async move {
                // No reader means no events; the UI shows its "no data" state.
                let Ok(reader) = WindowsStatsReader::new() else {
                    return;
                };
                let source = SystemStatsSource::new(reader, interval_ms, metrics);
                let mut sub = source.subscribe();
                while let Some(value) = sub.next().await {
                    if app.emit(&event, &value).is_err() {
                        break;
                    }
                }
            }));
        }
    }
    tasks
}

/// The sources need Windows APIs; other platforms start none.
#[cfg(not(windows))]
fn start_all(
    _app: &AppHandle,
    _deck: &Deck,
    _allowed: &dyn Fn(&DeckSource) -> bool,
) -> Vec<JoinHandle<()>> {
    Vec::new()
}
