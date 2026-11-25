//! [`StatsReader`] for Windows: CPU and RAM through windows-rs, GPU through `nvidia-smi`.

use easydeck_core::sources::system_stats::{parse_nvidia_smi, StatsReader, StatsSnapshot};
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Threading::GetSystemTimes;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Longest we wait for `nvidia-smi` before we kill it and give up on the GPU.
const GPU_TIMEOUT: Duration = Duration::from_secs(2);
/// A CPU reading needs at least this much elapsed system time (10 ms, in 100 ns ticks).
const MIN_CPU_DELTA_TICKS: u64 = 100_000;

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
    /// False after one `nvidia-smi` failure or timeout, so we stop spawning it every interval.
    gpu_available: bool,
}

impl WindowsStatsReader {
    pub fn new() -> windows::core::Result<Self> {
        Self::with_gpu(true)
    }

    fn with_gpu(gpu_available: bool) -> windows::core::Result<Self> {
        // CPU load is a delta between two reads; prime the first one.
        let last_times = system_times()?;
        Ok(Self {
            last_times,
            gpu_available,
        })
    }

    /// CPU load in percent since the previous call. `None` when the two samples are too
    /// close together to mean anything (for example the first read right after `new`).
    fn read_cpu(&mut self) -> Option<f64> {
        let (idle, total) = system_times().ok()?;
        let (last_idle, last_total) = std::mem::replace(&mut self.last_times, (idle, total));
        let total_delta = total.saturating_sub(last_total);
        if total_delta < MIN_CPU_DELTA_TICKS {
            return None;
        }
        let idle_delta = idle.saturating_sub(last_idle).min(total_delta);
        Some((1.0 - idle_delta as f64 / total_delta as f64) * 100.0)
    }

    fn read_gpu(&mut self) -> (Option<f64>, Option<f64>) {
        if !self.gpu_available {
            return (None, None);
        }
        let reading = run_nvidia_smi().and_then(|text| parse_nvidia_smi(&text));
        match reading {
            Some(r) => (r.load, r.temp),
            None => {
                self.gpu_available = false;
                (None, None)
            }
        }
    }
}

/// Runs `nvidia-smi` with fixed arguments and returns its stdout, or `None` on any failure
/// or when it runs past [`GPU_TIMEOUT`] (the child is then killed).
fn run_nvidia_smi() -> Option<String> {
    let mut child = Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    // Drain stdout on a thread so a full pipe can never block the child.
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let deadline = Instant::now() + GPU_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let buf = reader.join().ok()?;
    let status = status?;
    status
        .success()
        .then(|| String::from_utf8_lossy(&buf).into_owned())
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

    /// Reads CPU and RAM once. GPU reading is off, so no real program starts.
    #[test]
    fn reads_cpu_and_ram_in_range() {
        let mut r = WindowsStatsReader::with_gpu(false).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(250));
        let s = r.read();
        let cpu = s.cpu_load.unwrap();
        assert!((0.0..=100.0).contains(&cpu), "cpu {cpu}");
        let (used, total) = (s.ram_used.unwrap(), s.ram_total.unwrap());
        assert!(
            total > 0.0 && used > 0.0 && used <= total,
            "ram {used}/{total}"
        );
        assert!(s.gpu_load.is_none() && s.gpu_temp.is_none());
    }
}
