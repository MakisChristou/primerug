use rand::Rng;
use rug::Integer;
use std::fs::File;
use std::io::Write;

/// Compute the primorial p_m# (product of the first `m + 1` primes).
pub fn primorial(m: u64) -> Integer {
    let primes = generate_prime_table(m * m + 1);
    primes
        .iter()
        .take(m as usize + 1)
        .fold(Integer::from(1), |acc, &p| Integer::from(acc * p))
}

/// Generate all primes up to `limit` using a Sieve of Eratosthenes.
///
/// Ported from rieMiner (Pttn).
pub fn generate_prime_table(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return vec![];
    }

    let mut composites = vec![0u64; limit as usize / 128 + 1];

    let mut f = 3u64;
    while f * f <= limit {
        if composites[(f >> 7) as usize] & (1 << ((f >> 1) & 63)) == 0 {
            let mut m = (f * f) >> 1;
            while m <= limit >> 1 {
                composites[m as usize >> 6] |= 1 << (m & 63);
                m += f;
            }
        }
        f += 2;
    }

    let mut primes = vec![1, 2];
    let mut i = 1u64;
    while (i << 1) < limit {
        if composites[i as usize >> 6] & (1 << (i & 63)) == 0 {
            primes.push((i << 1) + 1);
        }
        i += 1;
    }
    primes
}

/// Compute the modular inverse of `primorial` mod each prime.
pub fn primorial_inverses(primorial: &Integer, primes: &[u64]) -> Vec<u64> {
    primes
        .iter()
        .map(|&p| {
            let modulus = Integer::from(p);
            primorial
                .invert_ref(&modulus)
                .map(|r| Integer::from(r).to_u64().unwrap())
                .unwrap_or(0)
        })
        .collect()
}

/// Generate a random integer with exactly `digits` decimal digits.
pub fn random_target(digits: u32, rng: &mut impl Rng) -> Integer {
    let mut s = String::with_capacity(digits as usize);
    // First digit 1-9 so the number has exactly `digits` digits
    s.push(char::from_digit(rng.gen_range(1..=9), 10).unwrap());
    for _ in 1..digits {
        s.push(char::from_digit(rng.gen_range(0..=9), 10).unwrap());
    }
    s.parse::<Integer>().unwrap()
}

/// Append found tuples to a file.
pub fn save_tuples(tuples: &[Integer], path: &str, tuple_len: usize) {
    let mut file = File::options()
        .create(true)
        .append(true)
        .open(path)
        .expect("cannot open tuple file");

    for tuple in tuples {
        writeln!(file, "{tuple_len}-tuple: {tuple}").expect("cannot write tuple");
    }
}
