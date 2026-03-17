use clap::Parser;
use rand::thread_rng;
use rug::Assign;
use rug::Integer;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

mod args;
mod config;
mod stats;
mod tools;

use args::Args;
use config::Config;
use stats::Stats;

const SIEVE_BITS: u32 = 25;
const SIEVE_SIZE: usize = 1 << SIEVE_BITS;
const SIEVE_WORDS: usize = SIEVE_SIZE / 64;

/// Pre-allocated GMP integer buffers to eliminate per-candidate heap allocations.
struct WorkBuffers {
    factors_to_eliminate: Vec<u32>,
    factors_table: Vec<u64>,
    candidate: Integer,
    offset_buf: Integer,
    fermat_base: Integer,
    fermat_exp: Integer,
}

impl WorkBuffers {
    fn new(pattern_len: usize, num_primes: usize) -> Self {
        Self {
            factors_to_eliminate: vec![0u32; pattern_len * num_primes],
            factors_table: vec![0u64; SIEVE_WORDS],
            candidate: Integer::new(),
            offset_buf: Integer::new(),
            fermat_base: Integer::new(),
            fermat_exp: Integer::new(),
        }
    }

    fn reset_sieve(&mut self) {
        self.factors_to_eliminate.fill(0);
        self.factors_table.fill(0);
    }
}

/// Fermat probable-prime test: 2^(n-1) ≡ 1 (mod n).
#[inline]
fn fermat(n: &Integer, base: &mut Integer, exp: &mut Integer) -> bool {
    base.assign(2);
    exp.assign(n);
    *exp -= 1u32;
    base.pow_mod_mut(exp, n).is_ok() && *base == 1
}

/// Test whether `n` starts a prime constellation matching `pattern`.
#[inline]
fn is_constellation(
    n: &Integer,
    pattern: &[u64],
    stats: &Stats,
    offset_buf: &mut Integer,
    base: &mut Integer,
    exp: &mut Integer,
) -> bool {
    stats.increment(0);

    for (i, &offset) in pattern.iter().enumerate() {
        offset_buf.assign(n + offset);
        if !fermat(offset_buf, base, exp) {
            return false;
        }
        stats.increment(i + 1);
    }
    true
}

/// Round `t` up to the next multiple of `primorial`.
fn next_primorial_multiple(t: &Integer, primorial: &Integer) -> Integer {
    let remainder = Integer::from(t % primorial);
    Integer::from(t + primorial) - remainder
}

/// Compute multiplicative-inverse multiples for sieve offset calculation.
#[inline]
fn compute_mi(inverses: &[u64], p: u64, i: usize) -> [u64; 4] {
    let m0 = inverses[i];
    let mut m1 = m0 << 1;
    if m1 >= p {
        m1 -= p;
    }
    let mut m2 = m1 << 1;
    if m2 >= p {
        m2 -= p;
    }
    let mut m3 = m1 + m2;
    if m3 >= p {
        m3 -= p;
    }
    [m0, m1, m2, m3]
}

/// Build the sieve: compute factor offsets and mark composite positions.
fn build_sieve(
    bufs: &mut WorkBuffers,
    target: &Integer,
    primes: &[u64],
    inverses: &[u64],
    config: &Config,
) {
    let m = config.m as usize;
    let tuple_size = config.pattern.len();
    let half_pattern = &config.half_pattern;
    let primorial = &config.primorial;

    let t2 = next_primorial_multiple(target, primorial);
    let first_candidate = Integer::from(&t2 + config.o);

    // Compute factor offsets for each prime >= p_m
    for (i, &p) in primes.iter().enumerate() {
        if i < m {
            continue;
        }

        let mi = compute_mi(inverses, p, i);
        let r = first_candidate.mod_u(p.try_into().unwrap()) as u64;
        let mut f_p = ((p - r) * inverses[i]) % p;

        bufs.factors_to_eliminate[tuple_size * i] = f_p as u32;

        for f in 1..tuple_size {
            let hp = mi[half_pattern[f] as usize];
            if f_p < hp {
                f_p += p;
            }
            f_p -= hp;
            bufs.factors_to_eliminate[tuple_size * i + f] = f_p as u32;
        }
    }

    // Mark composite positions using a write-combining cache
    let cache_size = 32usize;
    let mut cache = vec![0u32; cache_size];
    let mut cache_pos = 0usize;

    for (i, &p) in primes.iter().enumerate() {
        if i < m {
            continue;
        }
        for f in 0..tuple_size {
            let idx = i * tuple_size + f;
            while bufs.factors_to_eliminate[idx] < SIEVE_SIZE as u32 {
                let ent = bufs.factors_to_eliminate[idx];

                // Flush old cache entry to the sieve
                let old = cache[cache_pos];
                if old != 0 {
                    bufs.factors_table[(old >> 6) as usize] |= 1 << (old & 63);
                }
                cache[cache_pos] = ent;
                cache_pos = (cache_pos + 1) & (cache_size - 1);

                bufs.factors_to_eliminate[idx] += p as u32;
            }
            bufs.factors_to_eliminate[idx] -= SIEVE_SIZE as u32;
        }
    }

    // Flush remaining cache entries
    for &old in &cache {
        if old != 0 {
            bufs.factors_table[(old >> 6) as usize] |= 1 << (old & 63);
        }
    }
}

/// Iterate surviving sieve candidates and test for prime constellations.
fn test_candidates(bufs: &mut WorkBuffers, target: &Integer, config: &Config, stats: &Stats) {
    let primorial = &config.primorial;
    let t2 = next_primorial_multiple(target, primorial);
    let first_candidate = Integer::from(&t2 + config.o);

    if primorial >= target {
        eprintln!("Error: primorial is >= target. Pick a smaller primorial number.");
        process::exit(1);
    }

    for (word_idx, &sieve_word) in bufs.factors_table[..SIEVE_WORDS].iter().enumerate() {
        let mut survivors = !sieve_word;

        while survivors != 0 {
            let bit = survivors.trailing_zeros();
            let f = word_idx as u32 * 64 + bit;

            // candidate = primorial * f + first_candidate
            bufs.candidate.assign(primorial * f);
            bufs.candidate += &first_candidate;

            if is_constellation(
                &bufs.candidate,
                &config.pattern,
                stats,
                &mut bufs.offset_buf,
                &mut bufs.fermat_base,
                &mut bufs.fermat_exp,
            ) {
                println!("Found: {}", bufs.candidate);
                tools::save_tuples(
                    &[bufs.candidate.clone()],
                    "tuples.txt",
                    config.pattern.len(),
                );
                process::exit(0);
            }

            survivors &= survivors - 1;
        }
    }
}

/// Main loop for a single worker thread.
fn worker_loop(config: &Config, primes: &[u64], inverses: &[u64], stats: &Stats) {
    let mut bufs = WorkBuffers::new(config.pattern.len(), primes.len());
    let mut rng = thread_rng();

    loop {
        let target = tools::random_target(config.digits, &mut rng);

        bufs.reset_sieve();
        build_sieve(&mut bufs, &target, primes, inverses, config);
        test_candidates(&mut bufs, &target, config, stats);
    }
}

fn main() {
    let args = Args::parse();

    println!("Tuple Digits: {}", args.digits);
    println!("Primorial Number: {}", args.m);
    println!("Primorial Offset: {}", args.o);
    println!("Constellation Pattern: {}", args.pattern);
    println!("Prime Table Limit: {}", args.table_limit);
    println!("Stats Interval: {}s", args.stats_interval);
    println!("Threads: {}", args.threads);

    let primorial = tools::primorial(args.m);

    let config = Arc::new(Config::new(
        args.digits,
        &args.pattern,
        args.m,
        args.o,
        args.table_limit,
        args.threads,
        primorial.clone(),
    ));

    println!(
        "Generating prime table up to {} with Sieve of Eratosthenes...",
        args.table_limit
    );
    let primes = Arc::new(tools::generate_prime_table(config.prime_table_limit));

    println!("Calculating primorial inverse data...");
    let inverses = Arc::new(tools::primorial_inverses(&primorial, &primes));

    let stats = Arc::new(Stats::new(config.pattern.len()));

    println!("Done. Starting sieve/primality testing loop...");

    // Stats printing thread
    let stats_ref = Arc::clone(&stats);
    let interval = args.stats_interval;
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(interval));
        println!("{stats_ref}");
    });

    // Worker threads
    let mut handles = Vec::with_capacity(args.threads);
    for _ in 0..args.threads {
        let config = Arc::clone(&config);
        let primes = Arc::clone(&primes);
        let inverses = Arc::clone(&inverses);
        let stats = Arc::clone(&stats);

        handles.push(thread::spawn(move || {
            worker_loop(&config, &primes, &inverses, &stats);
        }));
    }

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }
}
