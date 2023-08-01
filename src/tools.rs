use rand::{self, Rng}; // 0.8.0
use rug::Integer;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Mul;

pub fn get_primorial(primorial_number: u64) -> Integer {
    let primes = generate_primetable((primorial_number * primorial_number) + 1);

    let mut primorial = Integer::from(1);

    for i in primes.iter().take(primorial_number as usize + 1) {
        primorial = primorial.mul(i);
    }
    primorial
}

// Generate all prime numbers up to a given limit using an efficient Sieve of Eratosthenis
// Original implementation by Pttn
// https://github.com/Pttn/rieMiner/blob/master/tools.cpp#L25
pub fn generate_primetable(prime_table_limit: u64) -> Vec<u64> {
    if prime_table_limit < 2 {
        return Vec::new();
    }

    let mut composite_table: Vec<u64> = Vec::new();

    composite_table.resize((prime_table_limit as usize) / 128 + 1, 0);

    let mut f = 3;

    while f * f <= prime_table_limit {
        if ((composite_table[(f >> 7) as usize]) & (1 << ((f >> 1) & 63))) != 0 {
            f += 2;
            continue;
        }

        let mut m = (f * f) >> 1;

        while m <= (prime_table_limit >> 1) {
            composite_table[(m as usize) >> 6] |= 1 << (m & 63);

            m += f;
        }
        f += 2;
    }

    // We have eliminated the composites
    let mut prime_table: Vec<u64> = vec![1, 2];

    let mut i = 1;

    while (i << 1) < prime_table_limit {
        if (composite_table[(i as usize) >> 6] & (1 << (i & 63))) == 0 {
            prime_table.push((i << 1) + 1);
        }
        i += 1;
    }
    prime_table
}

// Generate list of primorial inverses mod each prime in our prime table
pub fn get_primorial_inverses(primorial: &Integer, v: &Vec<u64>) -> Vec<u64> {
    let mut inverses = Vec::new();

    for i in v {
        let modulo = Integer::from(*i);
        let r = primorial.invert_ref(&modulo);

        let inverse = match r {
            Some(r) => Integer::from(r),
            None => Integer::from(0),
        };
        inverses.push(inverse.to_u64().expect("Cannot convert inverse to u64"));
    }
    inverses
}

// Return a random number as a String with d digits
pub fn get_difficulty_seed(d: u32) -> String {
    let mut t_str = String::from("");
    let mut rng = rand::thread_rng();

    for _i in 0..d {
        let digit: u32 = rng.gen_range(1..9);
        let s: String = digit.to_string();
        t_str.push_str(&s);
    }
    t_str
}

// Save tuple vector in a text file
pub fn save_tuples(tuples: &Vec<Integer>, tuple_file: &String, tuple_type: &usize) {
    let _ = File::create(tuple_file).expect("Cannot create output file");

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(tuple_file)
        .expect("Cannot open file");

    for tuple in tuples {
        match writeln!(file, "{}-tuple: {}", tuple_type, tuple) {
            Ok(_) => {}
            Err(e) => panic!("Could not write tuple file {}", e),
        }
    }
}
