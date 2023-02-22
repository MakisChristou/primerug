A prime k-tuple finder based on the rug Rust crate. The goal of this software is to search for large prime k-tuples. The code is heavily inspired by Pttn's [RieMiner](https://github.com/Pttn/rieMiner). For a detailed explanation on how the algorithm works see my explanation [here](https://makischristou.gitbook.io/primes/) and Pttn's original [writeup](https://riecoin.dev/en/Mining_Algorithm). Currently primerug can be considered only as a learning exercise for me to better understand Rust and how efficient sieving works for prime k-tuples and not a rieMiner replacement for breaking world records. Depending on the configuration it is currently 2-3 times slower than rieMiner with the exact same search parameters. So if breaking world records is your goal, you should use rieMiner or any other state-of-the-art siever/primality tester for the time being.


# Future plans
* Automatically select primorial, offset and primetable limit
* Performance improvements

# Build
First install Rust from [here](https://www.rust-lang.org/tools/install). Then you can build the project by simply typing:

```bash
cargo build --release
```


# Command-line arguments
Currently to configure primerug you have to supply cli arguments. All except the size of the tuple in decimal digits are optional. The default arguments are shown when running `primerug --help`.

```bash
$ primerug --help
A prime k-tuple finder based on the rug Rust crate

Usage: primerug [OPTIONS] --digits <DIGITS>

Options:
  -d, --digits <DIGITS>          Size of the tuple in decimal digits
  -m, --m <M>                    Primorial number [default: 3]
  -o, --o <O>                    Primorial offset [default: 97]
  -p, --pattern <PATTERN>        Desired pattern [default: "0, 4, 6, 10, 12, 16"]
  -t, --tablelimit <TABLELIMIT>  Desired pattern [default: 100000]
  -i, --interval <INTERVAL>      Stats interval [default: 5]
  -t, --threads <THREADS>        Threads [default: 1]
  -h, --help                     Print help
  -V, --version                  Print version
```

# Crates used
* [rug](https://crates.io/crates/rug) for bignum arithmetic
* [clap](https://crates.io/crates/clap) for argument parsing

# Example Usage
## Default options
Running with the default options is not really recommended as primerug would most likely use non-optimal settings and even a different constellation pattern than what you desire. But its a good way to easily run the program and get a feel of how it works before having to tinker with the settings.

```bash
$ primerug --digits 100
Tuple Digits: 100
Primorial Number: 3
Primorial Offset: 97
Constellation Pattern: 0, 4, 6, 10, 12, 16
Prime Table Limit: 100000
Stats Interval: 5
Threads: 1
Generating primetable of the first primes up to 100000 with sieve of Eratosthenes...
Calculating primorial inverse data...
Done, starting sieving/primality testing loop...
c/s: 1, r: inf (0, 0, 0, 0, 0, 0) eta: 584554049253 y
c/s: 83, r: 13.16 (38, 0, 0, 0, 0, 0) eta: 17 h
c/s: 100, r: 13.10 (84, 5, 0, 0, 0, 0) eta: 14 h
c/s: 93, r: 12.30 (122, 8, 0, 0, 0, 0) eta: 10 h
```

## Custom Options
The following is a way to run primerug in order to search for an 8-tuple that has over 200 digits.

```bash
$ primerug --digits 200 -m 50 -o 380284918609481 --pattern "0, 2, 6, 8, 12, 18, 20, 26" --tablelimit 16777216
Tuple Digits: 200
Primorial Number: 50
Primorial Offset: 380284918609481
Constellation Pattern: 0, 2, 6, 8, 12, 18, 20, 26
Prime Table Limit: 16777216
Stats Interval: 5
Threads: 1
Generating primetable of the first primes up to 16777216 with sieve of Eratosthenes...
Calculating primorial inverse data...
Done, starting sieving/primality testing loop...
c/s: 1, r: inf (0, 0, 0, 0, 0, 0, 0, 0) eta: 584554049253 y
c/s: 1500, r: 15.20 (592, 34, 4, 0, 0, 0, 0, 0) eta: 22 d
c/s: 1272, r: 14.97 (935, 59, 6, 0, 0, 0, 0, 0) eta: 22 d
c/s: 1125, r: 15.22 (1183, 74, 7, 1, 0, 0, 0, 0) eta: 29 d
c/s: 1095, r: 15.49 (1485, 95, 8, 1, 0, 0, 0, 0) eta: 35 d
c/s: 1192, r: 15.34 (2021, 134, 10, 3, 0, 0, 0, 0) eta: 29 d
c/s: 1032, r: 15.38 (2080, 138, 10, 3, 0, 0, 0, 0) eta: 35 d
c/s: 1277, r: 15.26 (3014, 193, 15, 4, 0, 0, 0, 0) eta: 26 d
```

## Attempt to break a world record
```bash
$ primerug --digits 400 -m 157 -o 114023297140211 --pattern "0, 2, 6, 8, 12, 18, 20, 26" --tablelimit 894144000 --threads 30
tern "0, 2, 6, 8, 12, 18, 20, 26" --tablelimit 894144000 --threads 30
Tuple Digits: 400
Primorial Number: 157
Primorial Offset: 114023297140211
Constellation Pattern: 0, 2, 6, 8, 12, 18, 20, 26
Prime Table Limit: 894144000
Stats Interval: 5
Threads: 30
Generating primetable of the first 894144000 primes with sieve of Eratosthenes...
Calculating primorial inverse data...
Done, starting sieving/primality testing loop...
c/s: 899, r: 25.64 (842, 37, 3, 0, 0, 0, 0, 0) eta: 6 y
c/s: 4227, r: 25.56 (4797, 206, 8, 0, 0, 0, 0, 0) eta: 1 y
c/s: 3627, r: 25.41 (6281, 261, 10, 0, 0, 0, 0, 0) eta: 1 y
c/s: 4804, r: 25.31 (9300, 386, 11, 0, 0, 0, 0, 0) eta: 1 y
c/s: 5224, r: 25.31 (11971, 500, 15, 0, 0, 0, 0, 0) eta: 1 y
c/s: 5316, r: 25.27 (14094, 588, 21, 0, 0, 0, 0, 0) eta: 362 d
c/s: 6018, r: 25.00 (18536, 771, 30, 0, 0, 0, 0, 0) eta: 293 d
c/s: 5917, r: 24.92 (20899, 869, 33, 0, 0, 0, 0, 0) eta: 290 d
c/s: 6544, r: 25.02 (25369, 1052, 42, 0, 0, 0, 0, 0) eta: 271 d
c/s: 6334, r: 25.09 (27520, 1137, 46, 1, 0, 0, 0, 0) eta: 287 d
c/s: 6888, r: 25.18 (32006, 1321, 57, 1, 0, 0, 0, 0) eta: 271 d
c/s: 6649, r: 25.13 (34657, 1419, 59, 1, 0, 0, 0, 0) eta: 277 d
c/s: 6945, r: 25.13 (38689, 1589, 65, 1, 0, 0, 0, 0) eta: 265 d
```

