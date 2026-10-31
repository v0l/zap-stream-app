use std::time::{Duration, Instant};

pub struct FrameStats {
    last: Instant,
    window_start: Instant,
    frames: Vec<f32>,
}

impl FrameStats {
    pub fn enabled() -> Option<Self> {
        if cfg!(target_os = "android") || std::env::var_os("ZAP_STREAM_FRAME_STATS").is_some() {
            Some(Self::new())
        } else {
            None
        }
    }

    fn new() -> Self {
        let now = Instant::now();
        Self {
            last: now,
            window_start: now,
            frames: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.frames
            .push(now.duration_since(self.last).as_secs_f32() * 1000.0);
        self.last = now;
        if now.duration_since(self.window_start) >= Duration::from_secs(5) {
            self.report();
            self.window_start = now;
        }
    }

    fn report(&mut self) {
        let mut t = std::mem::take(&mut self.frames);
        if t.is_empty() {
            return;
        }
        t.sort_by(|a, b| a.total_cmp(b));
        let q = |p: f32| t[((t.len() - 1) as f32 * p) as usize];
        let mean = t.iter().sum::<f32>() / t.len() as f32;
        log::info!(
            "frame_stats frames={} mean={:.2}ms p50={:.2}ms p95={:.2}ms p99={:.2}ms max={:.2}ms",
            t.len(),
            mean,
            q(0.5),
            q(0.95),
            q(0.99),
            q(1.0)
        );
    }
}

pub struct Section {
    name: &'static str,
    start: Instant,
}

impl Section {
    pub fn start(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for Section {
    fn drop(&mut self) {
        let ms = self.start.elapsed().as_secs_f32() * 1000.0;
        if ms > 8.0 {
            log::info!("slow_section {} {:.1}ms", self.name, ms);
        }
    }
}
