use rug::Integer;

#[derive(Clone)]
pub struct Config {
    pub digits: u32,
    pub pattern: Vec<u64>,
    pub half_pattern: Vec<u64>,
    pub m: u64,
    pub o: u64,
    pub prime_table_limit: u64,
    pub threads: usize,
    pub primorial: Integer,
    pub sieve_iterations: u32,
}

impl Config {
    pub fn new(
        digits: u32,
        pattern_str: &str,
        m: u64,
        o: u64,
        prime_table_limit: u64,
        threads: usize,
        primorial: Integer,
        sieve_iterations: u32,
    ) -> Self {
        let pattern = Self::parse_pattern(pattern_str);
        let half_pattern = Self::compute_half_pattern(&pattern);
        Self {
            digits,
            pattern,
            half_pattern,
            m,
            o,
            prime_table_limit,
            threads,
            primorial,
            sieve_iterations,
        }
    }

    fn parse_pattern(s: &str) -> Vec<u64> {
        s.split(',')
            .map(|p| p.trim().parse().expect("invalid pattern offset"))
            .collect()
    }

    fn compute_half_pattern(pattern: &[u64]) -> Vec<u64> {
        let mut half = vec![0u64];
        for w in pattern.windows(2) {
            half.push((w[1] - w[0]) / 2);
        }
        half
    }
}
