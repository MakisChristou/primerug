use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use rug::{Assign, Integer};
use std::collections::HashSet;
use args::Args;
use clap::Parser;


// My own stuff
mod tools;
mod constellation;
mod args;


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


fn is_constellation(n: &Integer, v: &Vec<u64>) -> bool
{
    let mut count = 0;
    for i in v
    {
        let c = n.add(i).into();

        if !fermat(&c)
        {
            // if count > v.len()-2
            // {
            //     println!("Found {}-tuple", v.len()-2)
            // }
            return false;
        }
        count += 1;
    }
    true
}


fn bruteforce_search(v: &Vec<u64>)
{
    for i in (5..10_000_000u64).step_by(2)
    {
        if is_constellation(&Integer::from(i), v)
        {
            println!("{}", i);
        }
    }

}


fn wheel_factorization(v: &Vec<u64>, t: &Integer, primorial: &Integer, offset: &Integer)
{
    // $$T^{'} = T + p_m\# - (T \; mod \; p_m\#)$$
    let t_prime: Integer = t.add(primorial).into();
    let ret: Integer = t_prime.clone() % primorial;
    let t_prime: Integer = t_prime.sub(ret).into();

    // Start from T^{'} since Integer division works only if exact
    let mut f: Integer = t_prime.div_exact_ref(&primorial).into();

    // println!("Searching with...");
    // println!("f: {}", f);
    // println!("primorial: {}", primorial);

    while true
    {
        let j: Integer = (primorial.mul(&f)).add(offset).into();
        if is_constellation(&j, &v)
        {
            println!("Found {}-tuple {}", v.len(), j);
        }
        f+=1;
    }
}


fn efficient_wheel_factorization_hashset(v: &Vec<u64>, t: &Integer, primorial: &Integer, offset: &Integer, primes: &Vec<u64>, inverses: &Vec<u64>)
{
    // T2 = T + p_m - (T % p_m)$
    let t_prime: Integer = t.add(primorial).into();
    let ret: Integer = t.clone() % primorial;
    let t_prime: Integer = t_prime.sub(ret).into();

    // Start from T2 since Integer division works only if exact
    // f = t2 / p_m
    let mut f: Integer = t_prime.div_exact_ref(&primorial).into();

    println!("f: {}", f);
    println!("primorial: {}", primorial);
    println!("t_prime: {}", t_prime);

    // Sieve
    let mut eliminated_factors: HashSet<Integer> = HashSet::new();

    // Counters
    let mut eliminated_count = 0;
    let mut primes_count = 0;
    let mut primality_tests = 0;

    let k_max = 50;

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
                eliminated_factors.insert(Integer::from(f_p));
                eliminated_count+=1;
                
                // Sieve out multiples of f_p
                for k in 0..k_max
                {
                    f_p += p;
                    eliminated_factors.insert(Integer::from(f_p));
                    eliminated_count+=1;
                }
            }
            i+=1;
        }
    }

    let f_max = 10000000000u64;
    
    let mut i = 0;

    println!("Primality Testing...");

    while i < f_max
    {
        if !eliminated_factors.contains(&Integer::from(&f))
        {
            // j = p_m * f + o
            let j: Integer = (primorial.mul(&f)).add(offset).into();

            // Fermat Test on j
            if is_constellation(&j, &v)
            {
                primes_count+=1;
                // println!("Found {}-tuple {}", v.len(), j);
            }
            primality_tests+=1;
        }
        f+=1;
        i+=1;
    }


    println!("Found {} primes, with {} primality tests, eliminated {}", primes_count, primality_tests, eliminated_count);
}


fn efficient_wheel_factorization_sieve(v: &Vec<u64>, t: &Integer, primorial: &Integer, offset: &Integer, primes: &Vec<u64>, inverses: &Vec<u64>, prime_table_limit: u64)
{
    // Counters
    let mut primes_count = 0;
    let mut primality_tests = 0;

    // T2 = T + p_m - (T % p_m)$
    let t_prime: Integer = t.add(primorial).into();
    let ret: Integer = t.clone() % primorial;
    let t_prime: Integer = t_prime.sub(ret).into();

    // println!("f: 0");
    // println!("primorial: {}", primorial);
    // println!("t_prime: {}", t_prime);
    // println!("Candidates of the form: p_m * f + o + T2");
    println!("Candidates of the form: {} * f + {} + {}", primorial, offset, t_prime);

    let sieve = get_eliminated_factors(primes, inverses, &t_prime, offset, v, prime_table_limit);

    println!("Sieve Size: {} MB", sieve.len()/1000000);

    let f_max = 10000000000u64; // Has to be lower than sieve size

    // How do we handle this? What if we want to keep searching?
    let f_max = sieve.len();
    
    let mut i = 0;

    println!("Primality Testing...");

    // Start from T2 since Integer division works only if exact
    // f = t2 / p_m
    let mut f: Integer = t_prime.div_exact_ref(&primorial).into();

    let mut f = 0;

    let t_prime_plus_offset: Integer = (&t_prime).add(offset).into();

    while i < f_max
    {
        if !sieve[f]
        {
            // j = p_m * f + o + T2
            let j: Integer = (primorial.mul(&Integer::from(f))).add(&t_prime_plus_offset).into();


            // Fermat Test on j
            if is_constellation(&j, &v)
            {
                primes_count+=1;
                // println!("Found {}-tuple {}", v.len(), j);
            }
            primality_tests+=1;
        }
        f+=1;
        i+=1;
    }

    println!("Found {} primes, with {} primality tests, eliminated {}", primes_count, primality_tests, f_max - primality_tests);
}


fn get_eliminated_factors(primes: &Vec<u64>, inverses: &Vec<u64>, t_prime: &Integer, offset: &Integer, v: &Vec<u64>, prime_table_limit: u64) -> Vec<bool>
{
    let k_max = 1000;

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
                let mut f_p = ((p- (r as u64) ) * inverses[i]) % p;
                // eliminated_factors.insert(Integer::from(f_p));

                // println!("Eliminated {}", f_p);
                // How unsafe can you be
                sieve[f_p as usize] = true;

                // eliminated_count+=1;
                
                // Sieve out multiples of f_p
                for k in 0..k_max
                {
                    f_p += p;
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


fn main()
{
    let args = Args::parse();

    // Chosen or default settings
    println!("Tuple Digits: {}", args.digits);
    println!("Primorial Number: {}", args.m);
    println!("Primorial Offset: {}", args.o);
    println!("Constellation Pattern: {}", args.pattern);
    println!("Prime Table Limit: {}", args.tablelimit);

    let d: u32 = args.digits; // Choose number of digits

    let constellation_pattern = tools::get_pattern_vector(args.pattern); // Choose pattern here

    let m: u64 = args.m; // Choose primorial number here

    let o: u64 = args.o; // Choose offset here

    let prime_table_limit = args.tablelimit;

    let p_m = tools::get_primorial(m);

    let primes = tools::generate_primetable(prime_table_limit);

    let inverses = tools::get_primorial_inverses(&p_m, &primes);
    
    let t_str: String = tools::get_difficulty_seed(d);

    let t = Integer::from_str(&t_str).unwrap();

    efficient_wheel_factorization_sieve(&constellation_pattern, &t, &p_m, &Integer::from(o), &primes, &inverses, prime_table_limit);
}
