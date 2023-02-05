use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use rug::{Assign, Integer};
use pbr::ProgressBar;
use clap::Parser;
use bit_vec::BitVec;
use std::process;
use std::time::Instant;

// My own stuff
mod tools;
mod args;
mod config;
mod stats;

use config::Config;
use args::Args;
use stats::Stats;

#[inline(always)]
// Fermat primality test
fn fermat(n: &Integer) -> bool
{
    let a = Integer::from(2);
    let n_minus_one = n.sub(Integer::from(1));

    // a = a^(n-1) % n
    let a = a.pow_mod(&n_minus_one, &n).unwrap();

    // a == 1?
    a == 1
}

#[inline(always)]
fn is_constellation(n: &Integer, v: &Vec<u64>, miner_stats: &mut Stats) -> bool
{

    miner_stats.tuple_counts[0]+=1;

    // Check each pattern offset for primality
    for (index, offset) in v.iter().enumerate()
    {
        // n + offset
        let c = n.add(offset).into();

        if !fermat(&c)
        {
            return false;
        }
        // Update Tuple Stats
        // index+1 because we don't update the candidates
        miner_stats.tuple_counts[index+1]+=1;
    }
    true
}

fn wheel_factorization(v: &Vec<u64>, t: &Integer, primorial: &Integer, offset: &Integer, primes: &Vec<u64>, inverses: &Vec<u64>, prime_table_limit: u64) -> Vec<Integer>
{
    // Counters
    let mut primes_count = 0;
    let mut primality_tests = 0;

    // T2 = T + p_m - (T % p_m)
    let t_prime: Integer = (t + primorial).into();
    let ret: Integer = (t.clone() % (primorial)).into();
    let t_prime: Integer = (t_prime - ret.clone()).into();


    // Add check that primorial < t
    if primorial >= t
    {
        println!("Pick Smaller primorial number");
        return Vec::new();
    }

    // println!("Candidates of the form: p_m * f + o + T2");
    println!("Candidates of the form: {} * f + {} + {}", primorial, offset, t_prime);

    let sieve = get_eliminated_factors_bitwise(primes, inverses, &t_prime, offset, v, prime_table_limit);

    println!("sieve.len() = {}", sieve.len());
    println!("Sieve Size: {} MB", sieve.len()/(1_000_000));

    // How do we handle this? What if we want to keep searching?
    let f_max = sieve.len();

    println!("Primality Testing...");

    let mut miner_stats = Stats::new(v.len());

    let t_prime_plus_offset: Integer = (&t_prime).add(offset).into();

    let mut tuples: Vec<Integer> = Vec::new();

    for i in 0..sieve.len()*8
    {
        let eliminated = get_bit(&sieve, i as u64);

        // Hardcode Stats interval for now
        if i % 1000000 == 0
        {
            println!("{}", miner_stats.get_human_readable_stats());            
        }

        if eliminated != 1
        {
            // j = p_m * f + o + T2
            let j: Integer = (primorial.mul(&Integer::from(i))).add(&t_prime_plus_offset).into();

            // Fermat Test on j
            if is_constellation(&j, &v, &mut miner_stats)
            {
                primes_count+=1;

                tuples.push(j);

                // Save them as we go, just in case
                tools::save_tuples(&tuples, &String::from("tuples.txt"), &v.len());
            }
            primality_tests+=1;
        }
    }

    println!("Found {} tuples, with {} primality tests, eliminated {}", primes_count, primality_tests, f_max - primality_tests);

    tuples
    

}


fn get_t2(t: &Integer, primorial: &Integer) -> Integer
{
    // T2 = T + p_m - (T % p_m)
    let t_prime: Integer = (t + primorial).into();
    let ret: Integer = (t.clone() % (primorial)).into();
    let t_prime: Integer = (t_prime - ret.clone()).into();

    return t_prime;
}

fn wheel_factorization_rieminer(factors_table: &Vec<u64>, print_stats_every_seconds: &usize, miner_stats: &mut Stats, i: &mut usize , m: &u64, v: &Vec<u64>, t: &Integer, primorial: &Integer, offset: &Integer, primes: &Vec<u64>, inverses: &Vec<u64>, prime_table_limit: u64) -> Vec<Integer>
{
    // Sieve size, should be the same always
    let sieve_bits = 25;
    let sieve_size = 1 << sieve_bits;
    let sieve_words: usize = sieve_size/64;

    
    let t_prime = get_t2(t, primorial);

    // Add check that primorial < t
    if primorial >= t
    {
        println!("Pick Smaller primorial number");
        process::exit(0x0);
    }

    // first_candidate = T2 + o
    let first_candidate: Integer = (&t_prime).add(offset).into();

    let mut tuples: Vec<Integer> = Vec::new();
    let mut factor_offsets: Vec<u64> = Vec::new();

    // Remove multiples of f_p
    for b in 0..sieve_words
    {
        // Get sieve word
        let mut sieve_word: u64 = factors_table[b];

        // Bitwise not
        sieve_word = !sieve_word;

        // Eliminate multiples of f_p
        while sieve_word != 0
        {
            let n_eliminated_until_next: u32  = sieve_word.trailing_zeros();
            let candidate_index = ((b as u32)*64 + n_eliminated_until_next);

            factor_offsets.push(candidate_index as u64); // this holds all the f's that will be tested later on

            sieve_word &= sieve_word - 1;
        }
    }


    let mut iterations_per_second = 0;
    
    for f in factor_offsets
    {
        let start = Instant::now();
    
        // Print Stats for user selected interval
        if iterations_per_second!= 0 && (*i % (print_stats_every_seconds*iterations_per_second) == 0)
        {
            println!("{}", miner_stats.get_human_readable_stats());
        }

        // t = p_m * f + first_candidate
        let t: Integer = (primorial.mul(&Integer::from(f))).add(&first_candidate).into();

        // Fermat Test on j
        if is_constellation(&t, &v, miner_stats)
        {
            println!("Found: {}", t);

            tuples.push(t);

            // Save them as we go, just in case
            tools::save_tuples(&tuples, &String::from("tuples.txt"), &v.len());
            
            process::exit(0x0);
        }
        
        *i+=1;

        // Calculate iteration time
        let iteration_time = start.elapsed();
        iterations_per_second = 1_000_000_000/(iteration_time.as_nanos() as usize);

    }
    tuples
}

fn get_eliminated_factors_boolset(primes: &Vec<u64>, inverses: &Vec<u64>, t_prime: &Integer, offset: &Integer, v: &Vec<u64>, prime_table_limit: u64) -> Vec<bool>
{
    let k_max = 8000;

    let sieve_size = prime_table_limit +  prime_table_limit * k_max; // This has to be the same as prime_table limit * k_max

    let mut sieve = Vec::new();

    sieve.resize(sieve_size as usize, false);

    let t_prime_plus_offset: Integer = (&t_prime).add(offset).into();

    println!("Sieving...");

    let mut i = 0;
    for p in primes
    {   
        // Don't panic (I am sure there is a better way to do this)
        if *p != 0
        {
            for c_i in v
            {
                // (T2 + o + c_i)
                let t_prime_plus_offset_plus_c_i: Integer = (&t_prime_plus_offset).add(c_i).into();

                // ((T2 + o + c_i) % p)
                let r = t_prime_plus_offset_plus_c_i.mod_u((*p).try_into().unwrap());

                // f_p = ((p - ((T2 + o + c_i) % p))*p_m_inverse) % p
                let mut f_p = (((*p)- (r as u64) ) * inverses[i]) % (*p);
                // eliminated_factors.insert(Integer::from(f_p));

                // println!("Eliminated {}", f_p);
                // How unsafe can you be
                sieve[f_p as usize] = true;

                // eliminated_count+=1;
                
                // Sieve out multiples of f_p
                for k in 0..k_max
                {
                    f_p += (*p);
                    sieve[f_p as usize] = true;
                    // eliminated_factors.insert(Integer::from(f_p));
                    // eliminated_count+=1;
                }
            }
            i+=1;
        }
    }

    sieve
}

#[inline(always)]
fn get_bit(sieve: &Vec<u8>, i: u64) -> u8
{
    // Find which byte this bit is on
    let byte = i>>3;

    // Find the bit's position whithin the byte
    let position = i%8;

    // Shift left so value becomes LSB
    let shifted = sieve[byte as usize]>>(8-position);

    // Ignore/Mask higher values
    let value = shifted & 0x1;

    return value;
}

#[inline(always)]
fn set_bit(sieve: &mut Vec<u8>, i: u64, value: u8)
{
    // Find which byte this bit is on
    let byte = i>>3;

    // Find the bit's position whithin the byte
    let position = i%8;

    // Mask value's upper bits
    let value = value & 0x1; //0x00000001

    // Align bit with position
    let value = value <<(8-position); // 0x00010000

    // Set bit
    sieve[byte as usize] |= value;
}


#[inline(always)]
fn get_64_bit(sieve: &Vec<u64>, i: u64) -> u64
{
    // Find which byte this bit is on
    let byte = i/64;

    // Find the bit's position whithin the byte
    let position = i%64;

    // Shift left so value becomes LSB
    let shifted = sieve[byte as usize]>>(64-position);

    // Ignore/Mask higher values
    let value: u64 = shifted & 0x1;

    return value;
}

fn get_eliminated_factors_bitwise(primes: &Vec<u64>, inverses: &Vec<u64>, t_prime: &Integer, offset: &Integer, v: &Vec<u64>, prime_table_limit: u64) -> Vec<u8>
{
    let k_max = 1_000;

    let sieve_size = prime_table_limit +  prime_table_limit * k_max; // This has to be the same as prime_table limit * k_max

    let mut sieve = Vec::new();

    sieve.resize((((sieve_size/8)+1) as usize).try_into().unwrap(), 0);

    let t_prime_plus_offset: Integer = (&t_prime).add(offset).into();

    println!("Sieving...");

    let total = primes.len() * v.len() * k_max as usize;
    let mut pb = ProgressBar::new(total as u64);


    let mut i = 0;
    for p in primes
    {   

        if i % 1000 == 0
        {
            pb.add(1000*(v.len() as u64)*k_max);
        }

        // Don't panic (I am sure there is a better way to do this)
        if *p != 0
        {
            for c_i in v
            {
                // (T2 + o + c_i)
                let t_prime_plus_offset_plus_c_i: Integer = (&t_prime_plus_offset).add(c_i).into();

                // ((T2 + o + c_i) % p)
                let r = t_prime_plus_offset_plus_c_i.mod_u((*p).try_into().unwrap());

                // f_p = ((p - ((T2 + o + c_i) % p))*p_m_inverse) % p
                let mut f_p = (((*p)- (r as u64) ) * inverses[i]) % (*p);

                // Sieve out f_p
                set_bit(&mut sieve, f_p, 1);

                // Sieve out multiples of f_p
                for k in 0..k_max
                {
                    f_p += (*p);
                    set_bit(&mut sieve, f_p, 1);
                }
            }
            i+=1;
        }
    }
    sieve
}

fn get_eliminated_factors(primes: &Vec<u64>, inverses: &Vec<u64>, t_prime: &Integer, offset: &Integer, v: &Vec<u64>, prime_table_limit: u64) -> BitVec
{
    let k_max = 10;

    let sieve_size = prime_table_limit +  prime_table_limit * k_max; // This has to be the same as prime_table limit * k_max

    let mut sieve = BitVec::from_elem(sieve_size as usize, false);

    let t_prime_plus_offset: Integer = (&t_prime).add(offset).into();

    println!("Sieving...");

    let mut i = 0;
    for p in primes
    {   
        // Don't panic (I am sure there is a better way to do this)
        if *p != 0
        {
            for c_i in v
            {
                // (T2 + o + c_i)
                let t_prime_plus_offset_plus_c_i: Integer = (&t_prime_plus_offset).add(c_i).into();

                // ((T2 + o + c_i) % p)
                let r = t_prime_plus_offset_plus_c_i.mod_u((*p).try_into().unwrap());

                // f_p = ((p - ((T2 + o + c_i) % p))*p_m_inverse) % p
                let mut f_p = ((p- (r as u64) ) * inverses[i]) % p;

                // Sieve out f_p
                sieve.set(f_p as usize,true);
                
                // Sieve out multiples of f_p
                for k in 0..k_max
                {
                    f_p += p;
                    sieve.set(f_p as usize,true);
                }
            }
            i+=1;
        }
    }
    sieve
}

fn get_half_pattern(v: &Vec<u64>) -> Vec<u64>
{
    let mut half_pattern = Vec::new();

    half_pattern.push(0);

    for i in 0..v.len()-1
    {
        let distanse = v[i+1] - v[i];
        half_pattern.push(distanse/2);
    }
    half_pattern
}

fn get_mi(inverses: &Vec<u64>, p: &u64, i: usize) -> Vec<u64>
{
    let mut mi: Vec<u64> = Vec::new();
    mi.resize(4, 0);
    

    mi[0] = inverses[i];
    mi[1] = (mi[0] << 1); // mi[i] = (2*i*mi[0]) % p for i > 0.

    if mi[1] >= *p
    {
        mi[1] -= *p;
    }

    mi[2] = mi[1] << 1;

    if mi[2] >= *p
    {
        mi[2] -= *p;
    }

    mi[3] = mi[1] + mi[2];

    if mi[3] >= *p
    {
        mi[3] -= *p;
    }
    mi
}

fn add_to_sieve_cache(sieve: &mut Vec<u64>, sieve_cache: &mut Vec<u32>, pos: &mut usize, ent: u32)
{
    let old: u32 = sieve_cache[*pos];

    if old != 0
    {
        sieve[(old >> 6) as usize] |= (1 << (old & 63));
    }

    sieve_cache[*pos] = ent as u32;
    (*pos)+=1;
    (*pos) &= sieve_cache.len() - 1;
}

fn end_sieve_cache(sieve: &mut Vec<u64>, sieve_cache: &mut Vec<u32>)
{
    for i in 0..sieve_cache.len()
    {
        let old: u32 = sieve_cache[i];
        if old != 0
        {
            sieve[(old >> 6) as usize] |= (1 << (old & 63));
        }
    }
}

// Ported code from Pttn, wish I knew why it works
fn get_eliminated_factors_rieminer(t: &Integer, primorial: &Integer, m: &u64, primes: &Vec<u64>, inverses: &Vec<u64>, offset: &Integer, v: &Vec<u64>, prime_table_limit: u64) -> Vec<u64>
{
    let half_pattern = get_half_pattern(v);

    let sieve_bits = 25;

    let sieve_size = 1 << sieve_bits;

    let sieve_words: usize = sieve_size/64;

    let t_prime = get_t2(t, primorial);


    let mut factors_to_eliminate: Vec<u32> = Vec::new();
    factors_to_eliminate.resize(v.len() * primes.len() , 0);

    let mut factors_table: Vec<u64> = Vec::new();
    factors_table.resize(sieve_words, 0);

    // first_candidate = T2 + o
    let first_candidate: Integer = (&t_prime).add(offset).into();

    let tuple_size = v.len();

    let mut i = 0;

    for p in primes
    {
        // Don't panic (I am sure there is a better way to do this)
        if i >= (*m as usize)
        {
            // Calculate multiplicative inverse data
            let mi: Vec<u64> = get_mi(&inverses, p, i);

            // (first_candidate % p)
            let r = first_candidate.mod_u((*p).try_into().unwrap());

            // f_p = ((p - ((T2 + o) % p))*p_m_inverse) % p
            let mut f_p = (((*p)- (r as u64) ) * inverses[i]) % (*p);

            factors_to_eliminate[tuple_size*i] = f_p as u32;

            for f in 1..tuple_size
            {
                if f_p < mi[half_pattern[f] as usize]
                {
                    f_p+= (*p);
                }

                f_p -= mi[half_pattern[f] as usize];
                factors_to_eliminate[tuple_size * i + f] = f_p as u32;
            }
            
        }
        i+=1;
    }


    let mut sieve_cache_pos: usize = 0;
    let sieve_cache_size: usize = 32;
    let mut sieve_cache: Vec<u32> = Vec::new();
    sieve_cache.resize(sieve_cache_size, 0);

    let mut i = 0;

    // Process Sieve
    for p in primes
    {
        if i >= (*m) as usize
        {
            for f in 0..tuple_size
            {
                // Process Sieve (i.e. eliminate multiples of f_p)
                while factors_to_eliminate[i*tuple_size + f] < sieve_size as u32
                {
                    // Eliminate factor
                    add_to_sieve_cache(&mut factors_table, &mut sieve_cache, &mut sieve_cache_pos, factors_to_eliminate[i*tuple_size + f]);
                    
                    factors_to_eliminate[i*tuple_size + f] += (*p as u32);
                }
                factors_to_eliminate[i*tuple_size + f] -= (sieve_size as u32);
            }
            
        }
        i+=1;
    }

    end_sieve_cache(&mut factors_table, &mut sieve_cache);

    factors_table
}

fn main()
{
    let args = Args::parse();

    // Chosen or default settings
    println!("Tuple Digits: {}", args.digits);
    println!("Primorial Number: {}", args.m);
    println!("Primorial Offset: {}", args.o);
    println!("Constellation Pattern: {}", args.pattern);
    println!("Prime Table Limit: {}", args.tablelimit);
    println!("Stats Interval: {}", args.interval);

    // let config = Config::new(150, String::from("0, 2, 6, 8, 12, 18, 20, 26"), 58, 114023297140211, 7275957);

    let config = Config::new(args.digits, args.pattern, args.m, args.o, args.tablelimit);

    let p_m = tools::get_primorial(config.m);


    println!("Generating primetable of the first {} primes with sieve of Eratosthenes...", args.tablelimit);

    let primes = tools::generate_primetable_bitvector_half(config.prime_table_limit);

    println!("Calculating primorial inverse data...");
    let inverses = tools::get_primorial_inverses(&p_m, &primes);
    
    let mut i = 0;

    let mut miner_stats = Stats::new(config.constellation_pattern.len());

    println!("Done, starting sieving/primality testing loop...");

    // Loop until you find a tuple
    loop
    {
        // Here we generate a difficulty seed T
        let t_str: String = tools::get_difficulty_seed(config.d);
        let t = Integer::from_str(&t_str).unwrap();

        // Get factors f_p and their multiples
        let factors_table: Vec<u64> = get_eliminated_factors_rieminer(&t, &p_m, &config.m, &primes, &inverses, &Integer::from(config.o), &config.constellation_pattern, config.prime_table_limit);

        // Extract candidates and perform Fermat test
        wheel_factorization_rieminer(&factors_table, &args.interval, &mut miner_stats, &mut i, &config.m, &config.constellation_pattern, &t, &p_m, &Integer::from(config.o), &primes, &inverses, config.prime_table_limit);
    }
}
