//! [`StatsReader`] for Windows: CPU and RAM through `sysinfo`, GPU through `nvidia-smi`.

use easydeck_core::sources::system_stats::{parse_nvidia_smi, StatsReader, StatsSnapshot};
use std::os::windows::process::CommandExt;
use std::process::Command;
use sysinfo::{MemoryRefreshKind, System};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct WindowsStatsReader {
    sys: System,
    /// False once `nvidia-smi` is missing or fails, so we stop spawning it every interval.
    gpu_available: bool,
}

impl WindowsStatsReader {
    pub fn new() -> Result<Self, sysinfo::Error> {
        let mut sys = System::new()?;
        // CPU load is a delta between two refreshes; prime the first one.
        sys.refresh_cpu_usage();
        Ok(Self {
            sys,
            gpu_available: true,
        })
    }

    fn read_gpu(&mut self) -> (Option<f64>, Option<f64>) {
        if !self.gpu_available {
            return (None, None);
        }
        let out = Command::new("nvidia-smi")
            .args([
                "--query-gpu=utilization.gpu,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let reading = match out {
            Ok(o) if o.status.success() => parse_nvidia_smi(&String::from_utf8_lossy(&o.stdout)),
            _ => None,
        };
        match reading {
            Some(r) => (r.load, r.temp),
            None => {
                self.gpu_available = false;
                (None, None)
            }
        }
    }
}

impl StatsReader for WindowsStatsReader {
    fn read(&mut self) -> StatsSnapshot {
        self.sys.refresh_cpu_usage();
        self.sys
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let (gpu_load, gpu_temp) = self.read_gpu();
        StatsSnapshot {
            cpu_load: Some(f64::from(self.sys.global_cpu_usage())),
            cpu_temp: None,
            ram_used: Some(self.sys.used_memory() as f64),
            ram_total: Some(self.sys.total_memory() as f64),
            gpu_load,
            gpu_temp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads CPU and RAM once. Starts only `nvidia-smi`, and only if it is installed.
    #[test]
    fn reads_cpu_and_ram_in_range() {
        let mut r = WindowsStatsReader::new().unwrap();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let s = r.read();
        let cpu = s.cpu_load.unwrap();
        assert!((0.0..=100.0).contains(&cpu), "cpu {cpu}");
        let (used, total) = (s.ram_used.unwrap(), s.ram_total.unwrap());
        assert!(
            total > 0.0 && used > 0.0 && used <= total,
            "ram {used}/{total}"
        );
        if let Some(g) = s.gpu_load {
            assert!((0.0..=100.0).contains(&g));
        }
    }
}
