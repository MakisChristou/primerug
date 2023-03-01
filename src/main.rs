use clap::Parser;
use rug::Integer;
use std::collections::HashMap;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;
use std::process;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

// My own stuff
mod args;
mod config;
mod stats;
mod tools;

use args::Args;
use config::Config;
use stats::Stats;

#[inline(always)]
fn fermat(n: &Integer) -> bool {
    let a = Integer::from(2);
    let n_minus_one = n.sub(Integer::from(1));

    // a = a^(n-1) % n
    let a = a.pow_mod(&n_minus_one, &n).unwrap();

    // a == 1?
    a == 1
}

#[inline(always)]
fn is_constellation(n: &Integer, v: &Vec<u64>, miner_stats: &mut Stats) -> bool {
    miner_stats.tuple_counts[0] += 1;

    // Check each pattern offset for primality
    for (index, offset) in v.iter().enumerate() {
        // n + offset
        let c = n.add(offset).into();

        if !fermat(&c) {
            return false;
        }
        // Update Tuple Stats
        // index+1 because we don't update the candidates
        miner_stats.tuple_counts[index + 1] += 1;
    }
    true
}

fn get_t2(t: &Integer, primorial: &Integer) -> Integer {
    // T2 = T + p_m - (T % p_m)
    let t_prime: Integer = (t + primorial).into();
    let ret: Integer = (t.clone() % (primorial)).into();
    let t_prime: Integer = (t_prime - ret.clone()).into();
    t_prime
}

fn wheel_factorization(
    tx: &mpsc::Sender<(Vec<u64>, usize)>,
    factors_table: &Vec<u64>,
    miner_stats: &mut Stats,
    i: &mut usize,
    t: &Integer,
    thread_id: usize,
    config: &Config,
) -> Vec<Integer> {
    let offset = Integer::from(config.o);
    let v = &config.constellation_pattern;
    let primorial = &config.primorial;

    // Sieve size, should be the same always
    let sieve_bits = 25;
    let sieve_size = 1 << sieve_bits;
    let sieve_words: usize = sieve_size / 64;

    let t_prime = get_t2(t, primorial);

    // Add check that primorial < t
    if primorial >= t {
        println!("Pick Smaller primorial number");
        process::exit(0x0);
    }

    // first_candidate = T2 + o
    let first_candidate: Integer = (&t_prime).add(offset).into();

    let mut tuples: Vec<Integer> = Vec::new();
    let mut factor_offsets: Vec<u64> = Vec::new();

    // Remove multiples of f_p
    for (b, mut sieve_word) in factors_table[..sieve_words].iter().copied().enumerate() {

        // Bitwise not
        sieve_word = !sieve_word;

        // Eliminate multiples of f_p
        while sieve_word != 0 {
            let n_eliminated_until_next: u32 = sieve_word.trailing_zeros();
            let candidate_index = (b as u32) * 64 + n_eliminated_until_next;

            factor_offsets.push(candidate_index as u64); // this holds all the f's that will be tested later on

            sieve_word &= sieve_word - 1;
        }
    }

    // let mut iterations_per_second = 0;
    for f in factor_offsets {
        let cps = miner_stats.cps() as usize;
        let num_of_digits = cps.to_string().len() as i32;
        let rounded_number = (cps as f64 / 10.0f64.powi(num_of_digits - 1)) as usize
            * 10.0f64.powi(num_of_digits - 1) as usize;

        // Print Stats for user selected interval
        if (rounded_number != 0) && (*i % (rounded_number) == 0) {
            tx.send((miner_stats.get_tuple_counts(), thread_id))
                .unwrap();
            // println!("Sending {:?}", miner_stats.get_tuple_counts());
        }

        // t = p_m * f + first_candidate
        let t: Integer = (primorial.mul(&Integer::from(f)))
            .add(&first_candidate)
            .into();

        // Fermat Test on candidate t
        if is_constellation(&t, &v, miner_stats) {
            println!("Found: {}", t);

            tuples.push(t);

            tools::save_tuples(&tuples, &String::from("tuples.txt"), &v.len());

            process::exit(0x0);
        }
        *i += 1;
    }
    tuples
}

fn get_half_pattern(v: &Vec<u64>) -> Vec<u64> {
    let mut half_pattern = Vec::new();

    half_pattern.push(0);

    for i in 0..v.len() - 1 {
        let distanse = v[i + 1] - v[i];
        half_pattern.push(distanse / 2);
    }
    half_pattern
}

#[inline(always)]
fn get_mi(inverses: &Vec<u64>, p: &u64, i: usize) -> Vec<u64> {
    let mut mi: Vec<u64> = Vec::new();
    mi.resize(4, 0);

    mi[0] = inverses[i];
    mi[1] = mi[0] << 1; // mi[i] = (2*i*mi[0]) % p for i > 0.

    if mi[1] >= *p {
        mi[1] -= *p;
    }

    mi[2] = mi[1] << 1;

    if mi[2] >= *p {
        mi[2] -= *p;
    }

    mi[3] = mi[1] + mi[2];

    if mi[3] >= *p {
        mi[3] -= *p;
    }
    mi
}

#[inline(always)]
fn add_to_sieve_cache(sieve: &mut Vec<u64>, sieve_cache: &mut Vec<u32>, pos: &mut usize, ent: u32) {
    let old: u32 = sieve_cache[*pos];

    if old != 0 {
        sieve[(old >> 6) as usize] |= 1 << (old & 63);
    }

    sieve_cache[*pos] = ent as u32;
    (*pos) += 1;
    (*pos) &= sieve_cache.len() - 1;
}

#[inline(always)]
fn end_sieve_cache(sieve: &mut Vec<u64>, sieve_cache: &mut Vec<u32>) {
    for i in 0..sieve_cache.len() {
        let old: u32 = sieve_cache[i];
        if old != 0 {
            sieve[(old >> 6) as usize] |= 1 << (old & 63);
        }
    }
}

// Ported code from Pttn
fn get_eliminated_factors(
    factors_to_eliminate: &mut Vec<u32>,
    factors_table: &mut Vec<u64>,
    t: &Integer,
    primes: &Vec<u64>,
    inverses: &Vec<u64>,
    config: &Config,
) {
    let m = config.m;
    let offset = Integer::from(config.o);
    let v = &config.constellation_pattern;
    let primorial = &config.primorial;

    let half_pattern = get_half_pattern(v);

    let sieve_bits = 25;
    let sieve_size = 1 << sieve_bits;
    let t_prime = get_t2(t, primorial);

    // first_candidate = T2 + o
    let first_candidate: Integer = (&t_prime).add(offset).into();

    let tuple_size = v.len();

    let mut i = 0;

    for p in primes {
        // Don't panic (I am sure there is a better way to do this)
        if i >= (m as usize) {
            // Calculate multiplicative inverse data
            let mi: Vec<u64> = get_mi(&inverses, p, i);

            // (first_candidate % p)
            let r = first_candidate.mod_u((*p).try_into().unwrap());

            // f_p = ((p - ((T2 + o) % p))*p_m_inverse) % p
            let mut f_p = (((*p) - (r as u64)) * inverses[i]) % (*p);

            factors_to_eliminate[tuple_size * i] = f_p as u32;

            for f in 1..tuple_size {
                if f_p < mi[half_pattern[f] as usize] {
                    f_p += *p;
                }

                f_p -= mi[half_pattern[f] as usize];
                factors_to_eliminate[tuple_size * i + f] = f_p as u32;
            }
        }
        i += 1;
    }

    let mut sieve_cache_pos: usize = 0;
    let sieve_cache_size: usize = 32;
    let mut sieve_cache: Vec<u32> = Vec::new();
    sieve_cache.resize(sieve_cache_size, 0);

    let mut i = 0;

    // Process Sieve
    for p in primes {
        if i >= m as usize {
            for f in 0..tuple_size {
                // Process Sieve (i.e. eliminate multiples of f_p)
                while factors_to_eliminate[i * tuple_size + f] < sieve_size as u32 {
                    // Eliminate factor
                    add_to_sieve_cache(
                        factors_table,
                        &mut sieve_cache,
                        &mut sieve_cache_pos,
                        factors_to_eliminate[i * tuple_size + f],
                    );

                    factors_to_eliminate[i * tuple_size + f] += *p as u32;
                }
                factors_to_eliminate[i * tuple_size + f] -= sieve_size as u32;
            }
        }
        i += 1;
    }
    end_sieve_cache(factors_table, &mut sieve_cache);
}

// Blocks until it receives the tuple counts from each thread
fn receive_last_message(
    rx: &mpsc::Receiver<(Vec<u64>, usize)>,
    threads: usize,
) -> HashMap<usize, Vec<u64>> {
    let mut thread_messages: HashMap<usize, Vec<u64>> = HashMap::new();

    while thread_messages.len() != threads {
        loop {
            match rx.try_recv() {
                Ok(message) => {
                    thread_messages.insert(message.1, message.0);
                }
                Err(_) => break,
            }
        }
    }

    thread_messages
}

fn thread_loop(
    config: Arc<Config>,
    primes: Arc<Vec<u64>>,
    inverses: Arc<Vec<u64>>,
    tx: mpsc::Sender<(Vec<u64>, usize)>,
    thread_id: usize,
) {
    let sieve_bits = 25;
    let sieve_size = 1 << sieve_bits;
    let sieve_words: usize = sieve_size / 64;

    // Allocate memory for sieve
    let mut factors_to_eliminate: Vec<u32> =
        vec![0; config.constellation_pattern.len() * primes.len()];
    let mut factors_table: Vec<u64> = vec![0; sieve_words];

    let mut i = 0;

    let mut miner_stats = Stats::new(config.constellation_pattern.len());

    loop {
        // Here we generate a difficulty seed T
        let t_str: String = tools::get_difficulty_seed(config.d);
        let t = Integer::from_str(&t_str).expect("Invalid difficulty seed");

        // Reset Sieve
        factors_to_eliminate.iter_mut().for_each(|x| *x = 0);
        factors_table.iter_mut().for_each(|x| *x = 0);

        // Get factors f_p and their multiples (i.e. generate sieve)
        get_eliminated_factors(
            &mut factors_to_eliminate,
            &mut factors_table,
            &t,
            &primes,
            &inverses,
            &config,
        );

        // Test remaining candidates from the sieve
        wheel_factorization(
            &tx,
            &factors_table,
            &mut miner_stats,
            &mut i,
            &t,
            thread_id,
            &config,
        );
    }
}

fn main() {
    let args = Args::parse();

    // Chosen or default settings
    println!("Tuple Digits: {}", args.digits);
    println!("Primorial Number: {}", args.m);
    println!("Primorial Offset: {}", args.o);
    println!("Constellation Pattern: {}", args.pattern);
    println!("Prime Table Limit: {}", args.tablelimit);
    println!("Stats Interval: {}", args.interval);
    println!("Threads: {}", args.threads);

    // let config = Config::new(150, String::from("0, 2, 6, 8, 12, 18, 20, 26"), 58, 114023297140211, 7275957);

    let p_m = tools::get_primorial(args.m);

    let config = Config::new(
        args.digits,
        args.pattern,
        args.m,
        args.o,
        args.tablelimit,
        args.threads,
        p_m.clone(),
    );
    let extra_config = config.clone();

    println!(
        "Generating primetable of the first primes up to {} with sieve of Eratosthenes...",
        args.tablelimit
    );
    let primes = tools::generate_primetable(config.prime_table_limit);

    println!("Calculating primorial inverse data...");
    let inverses = tools::get_primorial_inverses(&p_m, &primes);

    println!("Done, starting sieving/primality testing loop...");

    // Multiple producer, single consumer channel
    let (tx, rx) = mpsc::channel::<(Vec<u64>, usize)>();

    // For printing thread
    let print_stats_interval = (args.interval * 1000) as u64;
    let start_time = Instant::now();
    let threads = config.threads;

    // For worker threads
    let shared_config = Arc::new(config);
    let shared_primes = Arc::new(primes);
    let shared_inverses = Arc::new(inverses);

    let mut handles = Vec::new();

    // Stat printing thread
    thread::spawn(move || loop {
        let msgs = receive_last_message(&rx, threads);

        let cloned_pattern_size = extra_config.constellation_pattern.len();

        let total_stats = Stats::gen_total_stats(msgs, start_time.clone(), cloned_pattern_size);
        println!("{}", total_stats.get_human_readable_stats());

        thread::sleep(Duration::from_millis(print_stats_interval));
    });

    // Spawn worker threads
    for i in 0..threads {
        let tx_i = tx.clone();

        let shared_config_value = Arc::clone(&shared_config);
        let shared_primes_value = Arc::clone(&shared_primes);
        let shared_inverses_value = Arc::clone(&shared_inverses);

        let t = thread::spawn(move || {
            thread_loop(
                shared_config_value,
                shared_primes_value,
                shared_inverses_value,
                tx_i,
                i,
            );
        });

        handles.push(t);
    }

    // Wait for threads to finish
    for handle in handles {
        handle.join().expect("Error joining worker thread");
    }
}
