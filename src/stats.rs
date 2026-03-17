use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Thread-safe statistics using atomics. Workers call `increment()`,
/// the stats thread reads via `Display` — no channels or locking needed.
pub struct Stats {
    counts: Vec<AtomicU64>,
    start: Instant,
    pattern_len: usize,
}

impl Stats {
    pub fn new(pattern_len: usize) -> Self {
        let counts = (0..=pattern_len).map(|_| AtomicU64::new(0)).collect();
        Self {
            counts,
            start: Instant::now(),
            pattern_len,
        }
    }

    #[inline]
    pub fn increment(&self, index: usize) {
        self.counts[index].fetch_add(1, Ordering::Relaxed);
    }

    pub fn cps(&self) -> u64 {
        let elapsed = self.start.elapsed().as_secs();
        if elapsed > 0 {
            self.counts[0].load(Ordering::Relaxed) / elapsed
        } else {
            0
        }
    }

    fn ratio(&self) -> f64 {
        let singles = self.counts[0].load(Ordering::Relaxed) as f64;
        let twins = self.counts[1].load(Ordering::Relaxed) as f64;
        if twins > 0.0 {
            singles / twins
        } else {
            0.0
        }
    }

    fn eta_seconds(&self) -> f64 {
        let r = self.ratio();
        let cps = self.cps() as f64;
        if r == 0.0 || cps == 0.0 {
            0.0
        } else {
            r.powi(self.pattern_len as i32) / cps
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c/s: {}, r: {:.2} (", self.cps(), self.ratio())?;
        for i in 1..=self.pattern_len {
            let count = self.counts[i].load(Ordering::Relaxed);
            if i == self.pattern_len {
                write!(f, "{count}) ")?;
            } else {
                write!(f, "{count}, ")?;
            }
        }

        let eta = self.eta_seconds() as u64;
        match eta {
            0..=59 => write!(f, "eta: {eta} s"),
            60..=3599 => write!(f, "eta: {} min", eta / 60),
            3600..=86399 => write!(f, "eta: {} h", eta / 3600),
            86400..=31556951 => write!(f, "eta: {} d", eta / 86400),
            _ => write!(f, "eta: {} y", eta / 31556952),
        }
    }
}
