use clap::Parser;
use rand::thread_rng;
use rug::Assign;
use rug::Integer;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};

mod args;
mod config;
mod gpu_client;
mod stats;
mod tools;

use args::Args;
use config::Config;
use gpu_client::GpuClient;
use stats::Stats;
use rug::integer::Order;

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

/// Compute factor offsets for each prime (the expensive mod_u step).
/// This is the "presieve" — only needs to run once per target.
fn compute_factor_offsets(
    factors: &mut [u32],
    first_candidate: &Integer,
    primes: &[u64],
    inverses: &[u64],
    config: &Config,
) {
    let m = config.m as usize;
    let tuple_size = config.pattern.len();
    let half_pattern = &config.half_pattern;

    for (i, &p) in primes.iter().enumerate() {
        if i < m {
            continue;
        }

        let mi = compute_mi(inverses, p, i);
        let r = first_candidate.mod_u(p.try_into().unwrap()) as u64;
        let mut f_p = ((p - r) * inverses[i]) % p;

        factors[tuple_size * i] = f_p as u32;

        for f in 1..tuple_size {
            let hp = mi[half_pattern[f] as usize];
            if f_p < hp {
                f_p += p;
            }
            f_p -= hp;
            factors[tuple_size * i + f] = f_p as u32;
        }
    }
}

/// Mark composite positions in the sieve.
///
/// Hybrid approach:
/// - Small primes (< SEGMENT_SIZE): processed in L1-sized segments for cache locality
/// - Large primes (>= SEGMENT_SIZE): write-combining cache with prefetch for scattered marks
fn mark_composites(
    factors: &mut [u32],
    sieve: &mut [u64],
    primes: &[u64],
    config: &Config,
) {
    const SEGMENT_BITS: u32 = 18; // 2^18 = 256K positions = 32KB of sieve (fits L1)
    const SEGMENT_SIZE: u32 = 1 << SEGMENT_BITS;

    let m = config.m as usize;
    let tuple_size = config.pattern.len();
    let num_segments = SIEVE_SIZE as u32 / SEGMENT_SIZE;

    // Find the split point: first prime >= SEGMENT_SIZE
    let large_start = primes
        .iter()
        .position(|&p| p >= SEGMENT_SIZE as u64)
        .unwrap_or(primes.len());

    // Phase 1: Small primes — segmented for L1 cache locality
    let sieve_ptr = sieve.as_mut_ptr();
    for seg in 0..num_segments {
        let seg_end = (seg + 1) * SEGMENT_SIZE;

        for (i, &p) in primes[m..large_start].iter().enumerate() {
            let i = i + m;
            let p32 = p as u32;
            for f in 0..tuple_size {
                let idx = i * tuple_size + f;
                let mut pos = factors[idx];
                while pos < seg_end {
                    // SAFETY: pos < SIEVE_SIZE, so pos >> 6 < SIEVE_WORDS = sieve.len()
                    unsafe {
                        let word = &mut *sieve_ptr.add((pos >> 6) as usize);
                        *word |= 1u64 << (pos & 63);
                    }
                    pos += p32;
                }
                factors[idx] = pos;
            }
        }
    }

    // Phase 2: Large primes — write-combining cache with prefetch
    let cache_size = 32usize;
    let mut cache = [0u32; 32];
    let mut cache_pos = 0usize;

    for (i, &p) in primes[large_start..].iter().enumerate() {
        let i = i + large_start;
        let p32 = p as u32;
        for f in 0..tuple_size {
            let idx = i * tuple_size + f;
            while factors[idx] < SIEVE_SIZE as u32 {
                let ent = factors[idx];

                let old = cache[cache_pos];
                if old != 0 {
                    sieve[(old >> 6) as usize] |= 1u64 << (old & 63);
                }
                cache[cache_pos] = ent;
                cache_pos = (cache_pos + 1) & (cache_size - 1);

                #[cfg(target_arch = "x86_64")]
                unsafe {
                    _mm_prefetch(
                        sieve.as_ptr().add((ent >> 6) as usize) as *const i8,
                        _MM_HINT_T0,
                    );
                }

                factors[idx] += p32;
            }
            factors[idx] -= SIEVE_SIZE as u32;
        }
    }

    // Flush remaining cache entries
    for &old in &cache {
        if old != 0 {
            sieve[(old >> 6) as usize] |= 1u64 << (old & 63);
        }
    }

    // Carry over small prime offsets for next sieve iteration
    for (i, _) in primes[m..large_start].iter().enumerate() {
        let i = i + m;
        for f in 0..tuple_size {
            let idx = i * tuple_size + f;
            factors[idx] -= SIEVE_SIZE as u32;
        }
    }
}

/// Iterate surviving sieve candidates and test for prime constellations.
fn test_candidates(
    bufs: &mut WorkBuffers,
    first_candidate: &Integer,
    config: &Config,
    stats: &Stats,
) {
    let primorial = &config.primorial;

    for (word_idx, &sieve_word) in bufs.factors_table[..SIEVE_WORDS].iter().enumerate() {
        let mut survivors = !sieve_word;

        while survivors != 0 {
            let bit = survivors.trailing_zeros();
            let f = word_idx as u32 * 64 + bit;

            // candidate = primorial * f + first_candidate
            bufs.candidate.assign(primorial * f);
            bufs.candidate += first_candidate;

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

/// Collect sieve survivors and send to GPU for constellation testing.
fn test_candidates_gpu(
    bufs: &mut WorkBuffers,
    first_candidate: &Integer,
    config: &Config,
    stats: &Stats,
    gpu: &mut GpuClient,
    gpu_batch_size: u32,
    limb_count: u16,
    batch_limbs: &mut Vec<u32>,
    batch_candidates: &mut Vec<Integer>,
) {
    let primorial = &config.primorial;
    batch_limbs.clear();
    batch_candidates.clear();

    // Collect survivors from sieve into batch
    for (word_idx, &sieve_word) in bufs.factors_table[..SIEVE_WORDS].iter().enumerate() {
        let mut survivors = !sieve_word;
        while survivors != 0 {
            let bit = survivors.trailing_zeros();
            let f = word_idx as u32 * 64 + bit;

            bufs.candidate.assign(primorial * f);
            bufs.candidate += first_candidate;

            // Extract limbs in LSF u32 order
            let limbs = bufs.candidate.to_digits::<u32>(Order::Lsf);
            // Pad to fixed limb_count
            batch_limbs.extend_from_slice(&limbs);
            for _ in limbs.len()..limb_count as usize {
                batch_limbs.push(0);
            }
            batch_candidates.push(bufs.candidate.clone());

            survivors &= survivors - 1;

            // Submit when batch is full
            if batch_candidates.len() as u32 >= gpu_batch_size {
                submit_gpu_batch(
                    batch_limbs,
                    batch_candidates,
                    config,
                    stats,
                    gpu,
                    limb_count,
                );
            }
        }
    }

    // Submit remaining candidates
    if !batch_candidates.is_empty() {
        submit_gpu_batch(
            batch_limbs,
            batch_candidates,
            config,
            stats,
            gpu,
            limb_count,
        );
    }
}

fn submit_gpu_batch(
    batch_limbs: &mut Vec<u32>,
    batch_candidates: &mut Vec<Integer>,
    config: &Config,
    stats: &Stats,
    gpu: &mut GpuClient,
    limb_count: u16,
) {
    let num = batch_candidates.len() as u32;
    stats.increment_by(0, num as u64);

    match gpu.submit_batch(batch_limbs, &config.pattern, limb_count, num) {
        Ok(result) => {
            // Update per-round stats from GPU results
            for (i, &count) in result.round_counts.iter().enumerate() {
                stats.increment_by(i + 1, count as u64);
            }

            for &idx in &result.survivor_indices {
                if let Some(cand) = batch_candidates.get(idx as usize) {
                    println!("Found: {cand}");
                    tools::save_tuples(
                        &[cand.clone()],
                        "tuples.txt",
                        config.pattern.len(),
                    );
                    process::exit(0);
                }
            }
        }
        Err(e) => {
            eprintln!("GPU batch error: {e}");
        }
    }

    batch_limbs.clear();
    batch_candidates.clear();
}

/// Main loop for a GPU-accelerated worker thread.
fn worker_loop_gpu(
    config: &Config,
    primes: &[u64],
    inverses: &[u64],
    stats: &Stats,
    gpu_socket: &str,
    gpu_batch_size: u32,
) {
    let mut bufs = WorkBuffers::new(config.pattern.len(), primes.len());
    let mut rng = thread_rng();
    let sieve_chunk = Integer::from(&config.primorial * SIEVE_SIZE as u64);

    // Compute limb_count from digit count: digits → bits → u32 limbs, rounded up to 64
    let bits = (config.digits as f64 * std::f64::consts::LOG2_10).ceil() as u32 + 64;
    let limb_count = ((bits + 31) / 32).max(64) as u16;

    let mut gpu = GpuClient::connect(gpu_socket).unwrap_or_else(|e| {
        panic!("Failed to connect to GPU service at {gpu_socket}: {e}");
    });

    let mut batch_limbs = Vec::with_capacity(gpu_batch_size as usize * limb_count as usize);
    let mut batch_candidates = Vec::with_capacity(gpu_batch_size as usize);

    loop {
        let target = tools::random_target(config.digits, &mut rng);
        let t2 = next_primorial_multiple(&target, &config.primorial);
        let base_candidate = Integer::from(&t2 + config.o);

        if &config.primorial >= &target {
            eprintln!("Error: primorial is >= target. Pick a smaller primorial number.");
            process::exit(1);
        }

        bufs.factors_to_eliminate.fill(0);
        bufs.factors_table.fill(0);
        compute_factor_offsets(
            &mut bufs.factors_to_eliminate,
            &base_candidate,
            primes,
            inverses,
            config,
        );

        mark_composites(
            &mut bufs.factors_to_eliminate,
            &mut bufs.factors_table,
            primes,
            config,
        );
        test_candidates_gpu(
            &mut bufs,
            &base_candidate,
            config,
            stats,
            &mut gpu,
            gpu_batch_size,
            limb_count,
            &mut batch_limbs,
            &mut batch_candidates,
        );

        for iter in 1..config.sieve_iterations {
            bufs.factors_table.fill(0);
            mark_composites(
                &mut bufs.factors_to_eliminate,
                &mut bufs.factors_table,
                primes,
                config,
            );
            let iter_candidate = Integer::from(&sieve_chunk * iter) + &base_candidate;
            test_candidates_gpu(
                &mut bufs,
                &iter_candidate,
                config,
                stats,
                &mut gpu,
                gpu_batch_size,
                limb_count,
                &mut batch_limbs,
                &mut batch_candidates,
            );
        }
    }
}

/// Main loop for a single worker thread.
fn worker_loop(config: &Config, primes: &[u64], inverses: &[u64], stats: &Stats) {
    let mut bufs = WorkBuffers::new(config.pattern.len(), primes.len());
    let mut rng = thread_rng();
    let sieve_chunk = Integer::from(&config.primorial * SIEVE_SIZE as u64);

    loop {
        let target = tools::random_target(config.digits, &mut rng);
        let t2 = next_primorial_multiple(&target, &config.primorial);
        let base_candidate = Integer::from(&t2 + config.o);

        if &config.primorial >= &target {
            eprintln!("Error: primorial is >= target. Pick a smaller primorial number.");
            process::exit(1);
        }

        // Compute factor offsets once per target (the expensive mod_u presieve)
        bufs.factors_to_eliminate.fill(0);
        bufs.factors_table.fill(0);
        compute_factor_offsets(
            &mut bufs.factors_to_eliminate,
            &base_candidate,
            primes,
            inverses,
            config,
        );

        // First iteration: mark composites and test
        mark_composites(
            &mut bufs.factors_to_eliminate,
            &mut bufs.factors_table,
            primes,
            config,
        );
        test_candidates(&mut bufs, &base_candidate, config, stats);

        // Subsequent iterations: reuse carried-over factor offsets
        for iter in 1..config.sieve_iterations {
            bufs.factors_table.fill(0);
            mark_composites(
                &mut bufs.factors_to_eliminate,
                &mut bufs.factors_table,
                primes,
                config,
            );

            let iter_candidate = Integer::from(&sieve_chunk * iter) + &base_candidate;
            test_candidates(&mut bufs, &iter_candidate, config, stats);
        }
    }
}

fn main() {
    let args = Args::parse();

    // Parse pattern early so we can use it for auto-selection
    let pattern: Vec<u64> = args
        .pattern
        .split(',')
        .map(|p| p.trim().parse().expect("invalid pattern offset"))
        .collect();

    // Auto-select primorial number if not specified
    let m = if args.m == 0 {
        tools::auto_primorial_number(args.digits, args.sieve_iterations, SIEVE_SIZE as u64)
    } else {
        args.m
    };

    let primorial = tools::primorial(m);

    // Auto-select primorial offset if not specified
    let o = if args.o == 0 {
        let small_primes = tools::generate_prime_table(m * m + 1);
        tools::find_primorial_offset(&pattern, &small_primes, m)
    } else {
        args.o
    };

    let gpu_mode = !args.gpu_socket.is_empty();

    println!("Tuple Digits: {}", args.digits);
    println!("Primorial Number: {} (p{}#)", m, m);
    println!("Primorial Offset: {}", o);
    println!("Constellation Pattern: {}", args.pattern);
    println!("Prime Table Limit: {}", args.table_limit);
    println!("Stats Interval: {}s", args.stats_interval);
    println!("Threads: {}", args.threads);
    println!("Sieve Iterations: {}", args.sieve_iterations);
    if gpu_mode {
        println!("GPU Socket: {}", args.gpu_socket);
        println!("GPU Batch Size: {}", args.gpu_batch_size);
    }

    let config = Arc::new(Config::new(
        args.digits,
        &args.pattern,
        m,
        o,
        args.table_limit,
        args.threads,
        primorial.clone(),
        args.sieve_iterations,
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
    let gpu_socket = args.gpu_socket.clone();
    let gpu_batch_size = args.gpu_batch_size;
    let mut handles = Vec::with_capacity(args.threads);
    for _ in 0..args.threads {
        let config = Arc::clone(&config);
        let primes = Arc::clone(&primes);
        let inverses = Arc::clone(&inverses);
        let stats = Arc::clone(&stats);
        let gpu_socket = gpu_socket.clone();

        handles.push(thread::spawn(move || {
            if gpu_socket.is_empty() {
                worker_loop(&config, &primes, &inverses, &stats);
            } else {
                worker_loop_gpu(
                    &config,
                    &primes,
                    &inverses,
                    &stats,
                    &gpu_socket,
                    gpu_batch_size,
                );
            }
        }));
    }

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }
}
