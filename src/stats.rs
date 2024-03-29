use std::collections::HashMap;
use std::time::Instant;

pub struct Stats {
    pub pattern_size: usize,
    pub tuple_counts: Vec<u64>,
    duration: Instant,
}

impl Stats {
    pub fn new(pattern_size: usize) -> Stats {
        let mut tuple_counts = Vec::new();
        tuple_counts.resize(pattern_size + 1, 0);

        Stats {
            pattern_size,
            tuple_counts,
            duration: Instant::now(),
        }
    }

    // Second constructor
    pub fn gen_total_stats(
        msgs: HashMap<usize, Vec<u64>>,
        duration: Instant,
        pattern_size: usize,
    ) -> Stats {
        let mut total_counts = vec![0; pattern_size + 1];

        for msg in msgs {
            for (i, count) in msg.1.into_iter().enumerate() {
                total_counts[i] += count;
            }
        }

        Stats {
            pattern_size,
            tuple_counts: total_counts,
            duration,
        }
    }

    pub fn cps(&self) -> u64 {
        let elapsed: u64 = self.duration.elapsed().as_secs();

        if !self.tuple_counts.is_empty() && elapsed > 0 {
            return self.tuple_counts[0] / (elapsed);
        }
        0
    }

    // Ratio of tuples with size 1 vs tuples with size 2
    fn r(&self) -> f64 {
        if !self.tuple_counts.is_empty() {
            let single_tuples = self.tuple_counts[0] as f64;
            let twin_tuples = self.tuple_counts[1] as f64;

            let ratio: f64 = single_tuples / twin_tuples;
            return ratio;
        }
        0.0
    }

    // Eta to find tuple of desired length in seconds
    fn get_eta(&self) -> f64 {
        let r = self.r();
        let tuple_length = self.pattern_size as f64;
        let cps = self.cps() as f64;

        if r == 0.0 || cps == 0.0 {
            0.0
        } else {
            // r^{tuple_len/cps} = estimated duration to find a tuple in seconds
            f64::powf(r, tuple_length) / cps
        }
    }

    pub fn get_human_readable_stats(&self) -> String {
        let mut s = format!("c/s: {}, r: {:.2} (", self.cps(), self.r());

        // for (index, offset) in v.iter().enumerate()
        for (index, count) in self.tuple_counts.iter().enumerate() {
            if index == self.tuple_counts.len() - 1 {
                s.push_str(&format!("{}) ", count));
            } else if index > 0 {
                s.push_str(&format!("{}, ", count));
            }
        }

        s.push_str(&self.get_human_readable_eta());

        s
    }

    fn get_human_readable_eta(&self) -> String {
        let eta_in_seconds = self.get_eta() as u64;

        if eta_in_seconds < 60 {
            format!("eta: {} s", eta_in_seconds)
        } else if eta_in_seconds < 3600 {
            format!("eta: {} min", eta_in_seconds / 60)
        } else if eta_in_seconds < 86400 {
            format!("eta: {} h", eta_in_seconds / 3600)
        } else if eta_in_seconds < 31556952 {
            format!("eta: {} d", eta_in_seconds / 86400)
        } else {
            format!("eta: {} y", eta_in_seconds / 31556952)
        }
    }

    pub fn get_tuple_counts(&self) -> Vec<u64> {
        self.tuple_counts.clone()
    }
}
