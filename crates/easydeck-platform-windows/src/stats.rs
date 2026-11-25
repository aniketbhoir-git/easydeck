//! [`StatsReader`] for Windows: CPU and RAM through windows-rs, GPU through `nvidia-smi`.

use easydeck_core::sources::system_stats::{parse_nvidia_smi, StatsReader, StatsSnapshot};
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Threading::GetSystemTimes;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn filetime_ticks(t: FILETIME) -> u64 {
    (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
}

/// Returns (idle, kernel + user) system times. Kernel time already includes idle.
fn system_times() -> windows::core::Result<(u64, u64)> {
    let (mut idle, mut kernel, mut user) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    // SAFETY: the three pointers are valid, writable FILETIMEs that outlive the call.
    unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user))? };
    Ok((
        filetime_ticks(idle),
        filetime_ticks(kernel) + filetime_ticks(user),
    ))
}

/// Returns (used, total) physical memory in bytes.
fn memory_bytes() -> Option<(f64, f64)> {
    let mut m = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: `m` is a valid MEMORYSTATUSEX with `dwLength` set, as the API requires.
    unsafe { GlobalMemoryStatusEx(&mut m) }.ok()?;
    Some((
        (m.ullTotalPhys - m.ullAvailPhys) as f64,
        m.ullTotalPhys as f64,
    ))
}

pub struct WindowsStatsReader {
    /// Previous (idle, kernel + user) system times, in 100 ns ticks.
    last_times: (u64, u64),
    /// False once `nvidia-smi` is missing or fails, so we stop spawning it every interval.
    gpu_available: bool,
}

impl WindowsStatsReader {
    pub fn new() -> windows::core::Result<Self> {
        // CPU load is a delta between two reads; prime the first one.
        let last_times = system_times()?;
        Ok(Self {
            last_times,
            gpu_available: true,
        })
    }

    /// CPU load in percent since the previous call.
    fn read_cpu(&mut self) -> Option<f64> {
        let (idle, total) = system_times().ok()?;
        let (last_idle, last_total) = std::mem::replace(&mut self.last_times, (idle, total));
        let total_delta = total.saturating_sub(last_total);
        if total_delta == 0 {
            return Some(0.0);
        }
        let idle_delta = idle.saturating_sub(last_idle).min(total_delta);
        Some((1.0 - idle_delta as f64 / total_delta as f64) * 100.0)
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
        let cpu_load = self.read_cpu();
        let ram = memory_bytes();
        let (gpu_load, gpu_temp) = self.read_gpu();
        StatsSnapshot {
            cpu_load,
            cpu_temp: None,
            ram_used: ram.map(|r| r.0),
            ram_total: ram.map(|r| r.1),
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
        std::thread::sleep(std::time::Duration::from_millis(250));
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
