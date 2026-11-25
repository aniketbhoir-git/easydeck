//! Data sources: a named feed that pushes values to the deck (SPEC §5).

pub mod system_stats;

use std::collections::BTreeMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// One reading from a source: metric name to a number, or `None` when the metric is not
/// available on this PC (for example `gpu.load` without an NVIDIA card). Serializes as a JSON
/// object whose values are numbers or `null`.
pub type SourceValue = BTreeMap<String, Option<f64>>;

/// A live subscription. Dropping it stops the task that feeds it.
pub struct Subscription {
    rx: mpsc::Receiver<SourceValue>,
    task: JoinHandle<()>,
}

impl Subscription {
    pub(crate) fn new(rx: mpsc::Receiver<SourceValue>, task: JoinHandle<()>) -> Self {
        Self { rx, task }
    }

    /// Waits for the next value. `None` once the source has stopped.
    pub async fn next(&mut self) -> Option<SourceValue> {
        self.rx.recv().await
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A data feed. `subscribe` must run inside a tokio runtime.
pub trait Source {
    fn subscribe(&self) -> Subscription;
}
