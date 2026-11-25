//! The `system_stats` source: CPU, RAM and GPU numbers on a fixed interval.
//! The OS reads sit behind [`StatsReader`]; the Windows one lives in
//! `easydeck-platform-windows`.

use super::{Source, SourceValue, Subscription};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

pub const DEFAULT_INTERVAL_MS: u64 = 1000;
pub const MIN_INTERVAL_MS: u64 = 250;
/// Every metric the source can report, in the order used when the deck names none.
pub const ALL_METRICS: [&str; 6] = [
    "cpu.load",
    "cpu.temp",
    "gpu.load",
    "gpu.temp",
    "ram.used",
    "ram.total",
];

/// One read of the machine. `None` means the number is not available.
/// CPU and GPU load are percent, temperatures are degrees Celsius, RAM is in bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StatsSnapshot {
    pub cpu_load: Option<f64>,
    pub cpu_temp: Option<f64>,
    pub ram_used: Option<f64>,
    pub ram_total: Option<f64>,
    pub gpu_load: Option<f64>,
    pub gpu_temp: Option<f64>,
}

impl StatsSnapshot {
    fn get(&self, metric: &str) -> Option<f64> {
        match metric {
            "cpu.load" => self.cpu_load,
            "cpu.temp" => self.cpu_temp,
            "ram.used" => self.ram_used,
            "ram.total" => self.ram_total,
            "gpu.load" => self.gpu_load,
            "gpu.temp" => self.gpu_temp,
            _ => None,
        }
    }
}

/// Reads the machine once. May block (OS calls, a child process), so the source calls it on
/// the blocking pool.
pub trait StatsReader: Send + 'static {
    fn read(&mut self) -> StatsSnapshot;
}

/// GPU numbers from one `nvidia-smi` line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuReading {
    pub load: Option<f64>,
    pub temp: Option<f64>,
}

/// Parses the output of
/// `nvidia-smi --query-gpu=utilization.gpu,temperature.gpu --format=csv,noheader,nounits`.
/// Uses the first GPU. Returns `None` for empty or malformed output. A field that is
/// `[N/A]` or not a number becomes `None`; if both are missing the result is `None`.
pub fn parse_nvidia_smi(output: &str) -> Option<GpuReading> {
    let line = output.lines().map(str::trim).find(|l| !l.is_empty())?;
    let mut parts = line.split(',');
    let load = parse_field(parts.next()?);
    let temp = parse_field(parts.next()?);
    if parts.next().is_some() || (load.is_none() && temp.is_none()) {
        return None;
    }
    Some(GpuReading { load, temp })
}

fn parse_field(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Polls a [`StatsReader`] every interval and publishes the chosen metrics.
pub struct SystemStatsSource<R: StatsReader> {
    reader: Arc<Mutex<R>>,
    interval: Duration,
    metrics: Vec<String>,
}

impl<R: StatsReader> SystemStatsSource<R> {
    /// `interval_ms` and `metrics` come from the deck; `None` takes the defaults. An interval
    /// below [`MIN_INTERVAL_MS`] is raised to it. Metric names are kept as given; an unknown
    /// name reads as `null`.
    pub fn new(reader: R, interval_ms: Option<u64>, metrics: Option<Vec<String>>) -> Self {
        let ms = interval_ms
            .unwrap_or(DEFAULT_INTERVAL_MS)
            .max(MIN_INTERVAL_MS);
        Self {
            reader: Arc::new(Mutex::new(reader)),
            interval: Duration::from_millis(ms),
            metrics: metrics.unwrap_or_else(|| ALL_METRICS.iter().map(|m| m.to_string()).collect()),
        }
    }
}

impl<R: StatsReader> Source for SystemStatsSource<R> {
    fn subscribe(&self) -> Subscription {
        let (tx, rx) = mpsc::channel(4);
        let reader = Arc::clone(&self.reader);
        let metrics = self.metrics.clone();
        let interval = self.interval;
        let task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                let reader = Arc::clone(&reader);
                let read = tokio::task::spawn_blocking(move || {
                    reader.lock().unwrap_or_else(|e| e.into_inner()).read()
                })
                .await;
                let Ok(snap) = read else { return };
                let value: SourceValue = metrics.iter().map(|m| (m.clone(), snap.get(m))).collect();
                if tx.send(value).await.is_err() {
                    return;
                }
            }
        });
        Subscription::new(rx, task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    struct Fake(u32);
    impl StatsReader for Fake {
        fn read(&mut self) -> StatsSnapshot {
            self.0 += 1;
            StatsSnapshot {
                cpu_load: Some(self.0 as f64),
                ram_used: Some(2.0),
                ram_total: Some(8.0),
                ..Default::default()
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn values_arrive_each_interval() {
        let src = SystemStatsSource::new(Fake(0), Some(500), None);
        let mut sub = src.subscribe();
        let start = Instant::now();
        for n in 1..=3u32 {
            let v = sub.next().await.unwrap();
            assert_eq!(v["cpu.load"], Some(n as f64));
            assert!(start.elapsed() >= Duration::from_millis(500 * (n as u64 - 1)));
        }
        assert!(start.elapsed() >= Duration::from_millis(1000));
    }

    #[tokio::test(start_paused = true)]
    async fn missing_gpu_is_null_and_metrics_are_filtered() {
        let metrics = Some(vec!["gpu.load".to_string(), "ram.used".to_string()]);
        let src = SystemStatsSource::new(Fake(0), None, metrics);
        let mut sub = src.subscribe();
        let v = sub.next().await.unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v["gpu.load"], None);
        assert_eq!(v["ram.used"], Some(2.0));
        assert_eq!(
            serde_json::to_string(&v).unwrap(),
            r#"{"gpu.load":null,"ram.used":2.0}"#
        );
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_the_subscription_stops_the_task() {
        let src = SystemStatsSource::new(Fake(0), Some(250), None);
        let sub = src.subscribe();
        drop(sub);
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(Arc::strong_count(&src.reader), 1);
    }

    #[test]
    fn interval_is_clamped() {
        let src = SystemStatsSource::new(Fake(0), Some(10), None);
        assert_eq!(src.interval, Duration::from_millis(MIN_INTERVAL_MS));
    }

    #[test]
    fn parses_one_gpu() {
        let r = parse_nvidia_smi("37, 54\n").unwrap();
        assert_eq!(
            r,
            GpuReading {
                load: Some(37.0),
                temp: Some(54.0)
            }
        );
    }

    #[test]
    fn parses_first_of_many_gpus_and_crlf() {
        let r = parse_nvidia_smi("5, 40\r\n90, 80\r\n").unwrap();
        assert_eq!(r.load, Some(5.0));
    }

    #[test]
    fn na_field_is_none() {
        let r = parse_nvidia_smi("[N/A], 61").unwrap();
        assert_eq!(
            r,
            GpuReading {
                load: None,
                temp: Some(61.0)
            }
        );
        assert_eq!(parse_nvidia_smi("[N/A], [N/A]"), None);
    }

    #[test]
    fn missing_gpu_empty_or_malformed_is_none() {
        assert_eq!(parse_nvidia_smi(""), None);
        assert_eq!(parse_nvidia_smi("  \n"), None);
        assert_eq!(
            parse_nvidia_smi("NVIDIA-SMI has failed because it couldn't communicate"),
            None
        );
        assert_eq!(parse_nvidia_smi("12"), None);
        assert_eq!(parse_nvidia_smi("1, 2, 3"), None);
        assert_eq!(parse_nvidia_smi("abc, def"), None);
    }
}
