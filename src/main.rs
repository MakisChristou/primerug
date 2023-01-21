use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use rug::{Assign, Integer};


mod tools;
mod constellation;

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

fn is_constellation(n: &Integer, v: &Vec<u32>) -> bool
{
    let mut count = 0;
    for i in v
    {
        let c = n.add(i).into();

        if !fermat(&c)
        {
            if count > v.len()-2
            {
                println!("Found {}-tuple", v.len()-2)
            }
            return false;
        }
        count += 1;
    }
    true
}

fn bruteforce_search(v: &Vec<u32>)
{

    for i in (5..10_000_000u64).step_by(2)
    {
        if is_constellation(&Integer::from(i), v)
        {
            println!("{}", i);
        }
    }

}


fn wheel_factorization(v: &Vec<u32>, t: &Integer, primorial: &Integer, offset: &Integer)
{
    // $$T^{'} = T + p_m\# - (T \; mod \; p_m\#)$$
    let t_prime: Integer = t.add(primorial).into();
    let ret: Integer = t_prime.clone() % primorial;
    let t_prime: Integer = t_prime.sub(ret).into();

    // Start from T^{'} since Integer division works only if exact
    let mut f: Integer = t_prime.div_exact_ref(&primorial).into();

    println!("f: {}", f);
    println!("primorial: {}", primorial);

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

fn main()
{
    // let primes = tools::generate_primetable(100);
    // return;


    // Algorithm Steps
    // User Given: pattern, pattern offsets, difficulty
    // Calculate primetableLimit = (difficulty^6) / (2^(3*(pattern_size+7))
    // Generate primetable based on limit
    // Sieve bits = given
    // 
    // Pick the largest primorial based on sieve bits
    let constallation_pattern: Vec<u32> = vec![0, 4, 6, 10, 12, 16];

    let t_str = "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let digits = t_str.len();

    println!("Searching for Tuples with >= {} digits", digits);

    let t = Integer::from_str(t_str).unwrap();
    wheel_factorization(&constallation_pattern, &t, &tools::get_primorial(771), &Integer::from(145933845312371u64));

    // c1 = p_30 * 1 + 1091257 + 0
    // c2 = p_30 * 1 + 1091257 + 4
    // c3 = p_30 * 1 + 1091257 + 6
    // c4 = p_30 * 1 + 1091257 + 10
    // c5 = p_30 * 1 + 1091257 + 12
    // c6 = p_30 * 1 + 1091257 + 16

    // Prime P = prime_table[30]
    // Prime P+1 = prime_table[31]
    // Prime P+2 = prime_table[32]
    // Check if c1 is divisible by prime P+1, P+2, ..., prime_table_limit
    // Check if c2 is divisible by prime P+1, P+2, ..., prime_table_limit
    // Check if c3 is divisible by prime P+1, P+2, ..., prime_table_limit
    // Check if c4 is divisible by prime P+1, P+2, ..., prime_table_limit
    // Check if c5 is divisible by prime P+1, P+2, ..., prime_table_limit
    // Check if c6 is divisible by prime P+1, P+2, ..., prime_table_limit
 

}
