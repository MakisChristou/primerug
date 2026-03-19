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
        .fold(Integer::from(1), |acc, &p| Integer::from(acc * p as u64))
}

/// Generate all primes up to `limit` using a Sieve of Eratosthenes.
///
/// Ported from rieMiner (Pttn). Returns `Vec<u32>` — all primes up to 2^32
/// fit in u32, saving 50% memory vs u64 at large table limits.
pub fn generate_prime_table(limit: u64) -> Vec<u32> {
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

    let mut primes: Vec<u32> = vec![1, 2];
    let mut i = 1u64;
    while (i << 1) < limit {
        if composites[i as usize >> 6] & (1 << (i & 63)) == 0 {
            primes.push(((i << 1) + 1) as u32);
        }
        i += 1;
    }
    primes
}

/// Compute the modular inverse of `primorial` mod each prime.
pub fn primorial_inverses(primorial: &Integer, primes: &[u32]) -> Vec<u32> {
    primes
        .iter()
        .map(|&p| {
            let modulus = Integer::from(p);
            primorial
                .invert_ref(&modulus)
                .map(|r| Integer::from(r).to_u64().unwrap() as u32)
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

/// Auto-select the primorial number for a given digit count and sieve parameters.
///
/// Mirrors rieMiner's logic: choose the largest primorial such that
/// `primorial * sieve_iterations * sieve_size < 10^digits`.
/// This ensures candidates don't overflow the target digit range.
pub fn auto_primorial_number(digits: u32, sieve_iterations: u32, sieve_size: u64) -> u64 {
    let factor_max = sieve_iterations as u64 * sieve_size;

    // target_limit = 10^digits / factor_max
    let target = Integer::from(Integer::u_pow_u(10, digits));
    let limit = Integer::from(&target / factor_max);

    // Generate enough primes for primorial selection
    let primes = generate_prime_table(1_000_000);

    let mut primorial = Integer::from(1);
    for (i, &p) in primes.iter().enumerate() {
        if i == 0 {
            continue; // skip the "1" entry
        }
        let next = Integer::from(&primorial * p as u64);
        if next >= limit {
            return i as u64; // primorial index (for the primorial() function)
        }
        primorial = next;
    }
    primes.len() as u64 - 1
}

/// Find a valid primorial offset for the given pattern.
///
/// The offset `o` must satisfy: `o + d` is coprime to the primorial for every
/// pattern element `d`. This ensures no pattern element is trivially composite.
pub fn find_primorial_offset(pattern: &[u64], primes: &[u32], m: u64) -> u64 {
    find_primorial_offsets(pattern, primes, m, 1)[0]
}

/// Find `count` valid primorial offsets for the given pattern.
pub fn find_primorial_offsets(pattern: &[u64], primes: &[u32], m: u64, count: usize) -> Vec<u64> {
    let small_primes: Vec<u32> = primes[1..=m as usize].iter().copied().collect();
    let mut offsets = Vec::with_capacity(count);

    'outer: for o in 1u64.. {
        for &d in pattern {
            let candidate = o + d;
            for &p in &small_primes {
                if candidate % p as u64 == 0 {
                    continue 'outer;
                }
            }
        }
        offsets.push(o);
        if offsets.len() >= count {
            break;
        }
    }
    offsets
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
