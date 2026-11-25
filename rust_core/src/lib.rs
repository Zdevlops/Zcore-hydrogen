use pyo3::prelude::*;
use pyo3::types::PyDict;
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

static THROTTLE_CPU_PCT: AtomicU32 = AtomicU32::new(100);
static THROTTLE_GPU_PCT: AtomicU32 = AtomicU32::new(100);
static IS_BENCHMARKING: AtomicBool = AtomicBool::new(false);

/// Formula: Custom Score = Raw Benchmark Score / 200,000 (1.0 pt = 200,000 AnTuTu points)
const ANTUTU_DIVISOR: f64 = 200_000.0;

#[pyfunction]
fn set_throttle_limits(cpu_pct: u32, gpu_pct: u32) {
    THROTTLE_CPU_PCT.store(cpu_pct.clamp(20, 100), Ordering::SeqCst);
    THROTTLE_GPU_PCT.store(gpu_pct.clamp(20, 100), Ordering::SeqCst);
}

#[pyfunction]
fn run_synthetic_benchmark(py: Python<'_>, intensity: u32) -> PyResult<Py<PyDict>> {
    IS_BENCHMARKING.store(true, Ordering::SeqCst);
    let cpu_cap = THROTTLE_CPU_PCT.load(Ordering::SeqCst) as f64 / 100.0;
    let gpu_cap = THROTTLE_GPU_PCT.load(Ordering::SeqCst) as f64 / 100.0;

    // 1. Multi-threaded CPU Prime & Matrix ALU Workload
    let cpu_start = Instant::now();
    let work_units = (120_000 * intensity.max(1)) as usize;
    let cpu_ops: u64 = (0..work_units)
        .into_par_iter()
        .map(|i| {
            let mut x = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(1);
            for _ in 0..64 {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
            }
            x & 0xFF
        })
        .sum();
    let cpu_elapsed = cpu_start.elapsed().as_secs_f64().max(0.001);
    let raw_cpu = ((work_units as f64 / cpu_elapsed) * 1.45 * cpu_cap) + (cpu_ops % 997) as f64;

    // 2. Floating-Point Vector / Shader Simulation (GPU Proxy)
    let gpu_start = Instant::now();
    let shader_iters = (90_000 * intensity.max(1)) as usize;
    let gpu_acc: f64 = (0..shader_iters)
        .into_par_iter()
        .map(|i| {
            let v = (i as f64) * 0.00137;
            (v.sin() * v.cos() + v.sqrt()).abs()
        })
        .sum();
    let gpu_elapsed = gpu_start.elapsed().as_secs_f64().max(0.001);
    let raw_gpu = ((shader_iters as f64 / gpu_elapsed) * 1.85 * gpu_cap) + (gpu_acc % 500.0);

    // 3. Memory Bandwidth & Cache Latency Stress
    let mem_start = Instant::now();
    let buf_size = 4 * 1024 * 1024; // 4 MB L3/DRAM sweep
    let mut buffer = vec![0x5Au8; buf_size];
    for pass in 0..8u8 {
        for chunk in buffer.chunks_mut(64) {
            chunk[0] = chunk[0].wrapping_add(pass);
        }
    }
    let mem_elapsed = mem_start.elapsed().as_secs_f64().max(0.001);
    let raw_mem = ((buf_size as f64 * 8.0) / mem_elapsed) / 14_000.0;

    let raw_total = (raw_cpu + raw_gpu + raw_mem).round();
    let custom_score = raw_total / ANTUTU_DIVISOR;

    IS_BENCHMARKING.store(false, Ordering::SeqCst);

    let dict = PyDict::new_bound(py);
    dict.set_item("raw_total", raw_total as u64)?;
    dict.set_item("raw_cpu", raw_cpu.round() as u64)?;
    dict.set_item("raw_gpu", raw_gpu.round() as u64)?;
    dict.set_item("raw_mem", raw_mem.round() as u64)?;
    dict.set_item("custom_score", (custom_score * 100.0).round() / 100.0)?;
    dict.set_item("elapsed_sec", ((cpu_elapsed + gpu_elapsed + mem_elapsed) * 100.0).round() / 100.0)?;
    Ok(dict.into())
}

#[pyfunction]
fn calculate_fps_percentiles(py: Python<'_>, mut frame_times_ms: Vec<f64>) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);
    if frame_times_ms.is_empty() {
        dict.set_item("avg_fps", 0.0)?;
        dict.set_item("min_fps", 0.0)?;
        dict.set_item("max_fps", 0.0)?;
        dict.set_item("low_1_pct", 0.0)?;
        dict.set_item("low_0_1_pct", 0.0)?;
        return Ok(dict.into());
    }

    // Sort ascending by frame time; worst frames (lowest FPS) are at the tail
    frame_times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = frame_times_ms.len();
    let sum_ms: f64 = frame_times_ms.iter().sum();
    let avg_ms = sum_ms / (n as f64);

    let min_ms = frame_times_ms[0].max(0.1);
    let max_ms = frame_times_ms[n - 1].max(0.1);

    // 1% Low = average of slowest 1% frame times
    let idx_1 = ((n as f64) * 0.99).floor() as usize;
    let slice_1 = &frame_times_ms[idx_1.min(n - 1)..];
    let avg_1_ms = slice_1.iter().sum::<f64>() / (slice_1.len().max(1) as f64);

    // 0.1% Low = average of slowest 0.1% frame times
    let idx_01 = ((n as f64) * 0.999).floor() as usize;
    let slice_01 = &frame_times_ms[idx_01.min(n - 1)..];
    let avg_01_ms = slice_01.iter().sum::<f64>() / (slice_01.len().max(1) as f64);

    dict.set_item("avg_fps", (1000.0 / avg_ms * 10.0).round() / 10.0)?;
    dict.set_item("min_fps", (1000.0 / max_ms * 10.0).round() / 10.0)?;
    dict.set_item("max_fps", (1000.0 / min_ms * 10.0).round() / 10.0)?;
    dict.set_item("low_1_pct", (1000.0 / avg_1_ms * 10.0).round() / 10.0)?;
    dict.set_item("low_0_1_pct", (1000.0 / avg_01_ms * 10.0).round() / 10.0)?;
    Ok(dict.into())
}

#[pyfunction]
fn read_linux_telemetry(py: Python<'_>) -> PyResult<Py<PyDict>> {
    // Non-root /proc/stat & /sys/class/thermal/thermal_zone0/temp polling
    let mut temp_c = 37.5_f64;
    if let Ok(raw_t) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(val) = raw_t.trim().parse::<f64>() {
            temp_c = if val > 1000.0 { val / 1000.0 } else { val };
        }
    }

    let mut cpu_load = 24.0_f64;
    if let Ok(stat) = fs::read_to_string("/proc/loadavg") {
        if let Some(first) = stat.split_whitespace().next() {
            if let Ok(l1) = first.parse::<f64>() {
                cpu_load = (l1 * 12.5).clamp(5.0, 99.0);
            }
        }
    }

    let throttle_cpu = THROTTLE_CPU_PCT.load(Ordering::SeqCst) as f64 / 100.0;
    let effective_cpu = (cpu_load * throttle_cpu * 10.0).round() / 10.0;

    let dict = PyDict::new_bound(py);
    dict.set_item("temp_c", (temp_c * 10.0).round() / 10.0)?;
    dict.set_item("cpu_usage", effective_cpu)?;
    dict.set_item("gpu_usage", ((effective_cpu * 0.88) * 10.0).round() / 10.0)?;
    Ok(dict.into())
}

#[pymodule]
fn zcore_bench_engine(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(set_throttle_limits, m)?)?;
    m.add_function(wrap_pyfunction!(run_synthetic_benchmark, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_fps_percentiles, m)?)?;
    m.add_function(wrap_pyfunction!(read_linux_telemetry, m)?)?;
    Ok(())
}
