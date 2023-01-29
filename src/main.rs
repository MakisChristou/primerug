use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use rug::{Assign, Integer};
use std::collections::HashSet;

use pbr::ProgressBar;
extern crate pbr;

use pbr::ProgressBar as OtherProgressBar;

use clap::Parser;
use bit_vec::BitVec;

// My own stuff
mod tools;
mod constellation;
mod args;
mod config;
mod stats;

use config::Config;
use args::Args;
use stats::Stats;

// Based Fermat primality test
fn fermat(n: &Integer) -> bool
{
    let a = Integer::from(2);
    let n_minus_one = n.sub(Integer::from(1));

    // a = a^(n-1) % n
    let k = a.pow_mod(&n_minus_one, &n).unwrap();

    // a == 1?
    Integer::from(k) == Integer::from(1)
}

fn is_constellation(n: &Integer, v: &Vec<u64>, miner_stats: &mut Stats) -> bool
{
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
        miner_stats.tuple_counts[index]+=1;
    }
    true
}

fn wheel_factorization(v: &Vec<u64>, t: &Integer, primorial: &Integer, offset: &Integer, primes: &Vec<u64>, inverses: &Vec<u64>, prime_table_limit: u64) -> Vec<Integer>
{
    // Counters
    let mut primes_count = 0;
    let mut primality_tests = 0;

    // T2 = T + p_m - (T % p_m)$
    let t_prime: Integer = t.add(primorial).into();
    let ret: Integer = t.clone() % primorial;
    let t_prime: Integer = t_prime.sub(ret).into();

    // println!("Candidates of the form: p_m * f + o + T2");
    println!("Candidates of the form: {} * f + {} + {}", primorial, offset, t_prime);

    let sieve = get_eliminated_factors(primes, inverses, &t_prime, offset, v, prime_table_limit);

    println!("Sieve Size: {} MB", sieve.len()/(8*1_000_000));

    // How do we handle this? What if we want to keep searching?
    let f_max = sieve.len();

    println!("Primality Testing...");

    let mut miner_stats = Stats::new(v.len());

    let t_prime_plus_offset: Integer = (&t_prime).add(offset).into();

    let mut tuples: Vec<Integer> = Vec::new();

    let mut i = 0;



    for eliminated in sieve.iter().rev()
    {
        // Hardcode Stats interval for now
        if i % 200000 == 0
        {
            println!("{}", miner_stats.get_human_readable_stats());
            // if miner_stats.get_elapsed() > 0
            // {
            //     println!("tests/s: {}", (i)/miner_stats.get_elapsed());
            // }
            
        }

        if !eliminated
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
        i+=1;
    }

    println!("Found {} tuples, with {} primality tests, eliminated {}", primes_count, primality_tests, f_max - primality_tests);

    tuples
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

fn main()
{
    let args = Args::parse();

    // Chosen or default settings
    println!("Tuple Digits: {}", args.digits);
    println!("Primorial Number: {}", args.m);
    println!("Primorial Offset: {}", args.o);
    println!("Constellation Pattern: {}", args.pattern);
    println!("Prime Table Limit: {}", args.tablelimit);

    let config = Config::new(args.digits, args.pattern, args.m, args.o, args.tablelimit);

    let p_m = tools::get_primorial(config.m);

    let primes = tools::generate_primetable_bitvector_half(config.prime_table_limit);

    let inverses = tools::get_primorial_inverses(&p_m, &primes);
    
    let t_str: String = tools::get_difficulty_seed(config.d);

    let t = Integer::from_str(&t_str).unwrap();

    let tuples = wheel_factorization(&config.constellation_pattern, &t, &p_m, &Integer::from(config.o), &primes, &inverses, config.prime_table_limit);

    tools::save_tuples(&tuples, &String::from("tuples.txt"), &config.constellation_pattern.len());

}
