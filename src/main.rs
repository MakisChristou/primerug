use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use rug::{Assign, Integer};


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


fn wheel_factorization(v: &Vec<u32>, T: &Integer, primorial: &Integer, offset: &Integer)
{
    // $$T^{'} = T + p_m\# - (T \; mod \; p_m\#)$$
    let mut T_prime: Integer = T.add(primorial).into();
    let mut Ret: Integer = T_prime.clone() % primorial;
    let mut T_prime: Integer = T_prime.sub(Ret).into();

    // Start from T^{'} since Integer division works only if exact
    let mut f: Integer = T_prime.div_exact_ref(&primorial).into();

    println!("f: {}", f);
    println!("primorial: {}", primorial);

    while true
    {
        let j: Integer = (primorial.mul(&f)).add(offset).into();
        if is_constellation(&j, &v)
        {
            println!("{}", j);
        }
        f+=1;
    }
}

// Given the Target difficulty, choose a primorial
fn choose_primorial_number(T: Integer) -> u16
{
    return 3; // hardcode to 3 for now
}

// Returns the offcets for a given primorial
fn get_offsets(primorial_number: u16) -> Vec<u64>
{
    return Vec::new();
}

// Generate all prime numbers up to a given limit using an efficient Sieve of Eratosthenis
// Original implementation by Pttn
// https://github.com/Pttn/rieMiner/blob/master/tools.cpp#L25
fn generate_primetable(prime_table_limit: u64) -> Vec<u64>
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
        if ((composite_table[(f >> 7) as usize]) & (1 <<((f >> 1) & 63))) > 0
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
        if composite_table[(i as usize) >> 6] & (1 << (i & 63)) != 0
        {
            prime_table.push((i << 1) + 1);
        }
        i+=1;
    }
    prime_table
}

fn main()
{
    // let primes = generate_primetable(2147483648);
    // return;


    let constallation_pattern: Vec<u32> = vec![0, 4, 6, 10, 12, 16];

    let T_str = "1000000000000000000000000000000000";
    let digits = T_str.len();

    println!("Searching for Tuples with >= {} digits", digits);


    let T = Integer::from_str(T_str).unwrap();
    wheel_factorization(&constallation_pattern, &T, &Integer::from(2*3*5*7), &Integer::from(97));

}
