use std::ops::MulAssign;
use std::ops::Mul;
use rug::{Assign, Integer};
use rand::{self, Rng}; // 0.8.0

use bitvec::prelude::*;

// Given the Target difficulty, choose a primorial
pub fn choose_primorial_number(t: Integer) -> u16
{
    return 3; // hardcode to 3 for now
}

// Returns the offcets for a given primorial
pub fn get_offsets(primorial_number: u16) -> Vec<u64>
{
    return Vec::new();
}

pub fn get_primorial(primorial_number: u64) -> Integer
{
    let primes = generate_primetable((primorial_number*primorial_number)+1);

    let mut primorial = Integer::from(1);

    for i in primes.iter().take(primorial_number as usize + 1)
    {
        primorial = primorial.mul(i);
    }
    primorial
}

// Generate all prime numbers up to a given limit using an efficient Sieve of Eratosthenis
// Original implementation by Pttn
// https://github.com/Pttn/rieMiner/blob/master/tools.cpp#L25
pub fn generate_primetable(prime_table_limit: u64) -> Vec<u64>
{
    if prime_table_limit < 2
    {
        return Vec::new();
    }

    let mut composite_table: Vec<u64> = Vec::new();

    composite_table.resize((prime_table_limit as usize)/128 + 1, 0);

    let mut f = 3;

    while f*f <=prime_table_limit
    {
        if ((composite_table[(f >> 7) as usize]) & (1 <<((f >> 1) & 63))) != 0
        {
            f+=2;
            continue;
        }

        let mut m = (f*f) >> 1;

        while m <= (prime_table_limit >> 1)
        {
            composite_table[(m as usize) >> 6] |= 1 << (m & 63);

            m+=f;
        }
        f+=2;
    }

    // We have eliminated the composites
    let mut prime_table: Vec<u64> = Vec::from(vec![1,2]);

    let mut i = 1;

    while (i << 1) + 1 <= prime_table_limit
    {
        if (composite_table[(i as usize) >> 6] & (1 << (i & 63))) == 0
        {
            prime_table.push((i << 1) + 1);
        }
        i+=1;
    }
    prime_table
}

pub fn get_primorial_inverses(primorial: &Integer, v: &Vec<u64>) -> Vec<u64>
{
    let mut inverses = Vec::new();

    for i in v
    {
        let modulo = Integer::from(*i);
        let r = primorial.invert_ref(&modulo);

        let inverse = match r {
            Some(r) => Integer::from(r),
            None => Integer::from(0),
        };        
        inverses.push(inverse.to_u64().unwrap());
    }

    return inverses;
}

// Memory inneficient but readable sieve of eratosthenis
pub fn sieve_of_eratosthenes(n: u64) -> Vec<u64>
{
    if n < 2
    {
        return Vec::new();
    }

    let mut composite_table: Vec<bool> = Vec::new();
    composite_table.resize((n as usize)+1, true);

    let mut p = 2;

    while p*p <= n
    {
        if composite_table[p as usize] == true
        {
            let mut i = p*p;

            while i < n
            { 
                composite_table[i as usize] = false;

                i+=p;
            }
        }
        p+=1;
    }

    let mut primes = Vec::new();

    let mut index = 0;
    for i in composite_table
    {
        if i && index > 1
        {
            primes.push(index);
        }
        index+=1;
    }
    primes
}

pub fn get_difficulty_seed(d: u32) -> String
{
    let mut t_str = String::from("");
    let mut rng = rand::thread_rng();

    for i in (0..d)
    {
        let digit: u32 = rng.gen_range(1..9);
        let s: String = digit.to_string();
        t_str.push_str(&s);
    }
    t_str
}


pub fn get_pattern_vector(offsets: String) -> Vec<u64>
{
    let offsets: String = offsets.chars().filter(|c| !c.is_whitespace()).collect();

    let mut str_pattern = offsets.split(",");

    let mut pattern_vector: Vec<u64> = Vec::new();

    for o in str_pattern
    {
        pattern_vector.push(o.parse::<u64>().unwrap());
    }

    pattern_vector
}