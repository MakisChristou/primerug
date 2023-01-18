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
    for i in v
    {
        let c = n.add(i).into();

        if !fermat(&c)
        {
            return false;
        }
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


fn wheel_factorization(v: &Vec<u32>, start: &u128, end: &u128, primorial: u128, offset: u128)
{
    // Start
    let mut f = *start;

    // End
    let N = *end;

    // Wheel factorization go brrr
    while f < N/primorial
    {
        let j = (primorial*f) + offset;
        if is_constellation(&Integer::from(j), &v)
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



fn main() {

    let constallation_pattern: Vec<u32> = vec![0, 4, 6, 10, 12, 16];

    let T = Integer::from_str("10000000000000000000000000000000000000000000000000000000000000").unwrap();
    let primorial_numer = choose_primorial_number(T);

    wheel_factorization(&constallation_pattern, &1000_000_000_000_000_000u128, &1_000_000_000_000_000_000_000u128, 2*3*5*7, 97);

}
