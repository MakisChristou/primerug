# Primerug
A prime k-tuple finder based on the rug Rust crate. The code is heavily inspired by Pttn's [RieMiner](https://github.com/Pttn/rieMiner). For a detailed explanation on how the algorithm works see my explanation [here](https://makischristou.gitbook.io/primes/) and Pttn's original [writeup](https://riecoin.dev/en/Mining_Algorithm).

# Build
```bash
sudo apt install cargo
cargo build --release
```


# Command-line arguments
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
  -h, --help                     Print help
  -V, --version                  Print version
```

# Run with default options
```bash
primerug --digits 10
Tuple Digits: 10
Primorial Number: 3
Primorial Offset: 97
Constellation Pattern: 0, 4, 6, 10, 12, 16
Prime Table Limit: 100000
Candidates of the form: 30 * f + 97 + 1721455800
Sieving...
Sieve Size: 100 MB
Primality Testing...
Found 461 primes, with 65735927 primality tests, eliminated 34364073
```

# Using custom primorial number, offset, and constellation pattern
```bash
$ primerug --digits 10 -m 5 -o 88793 --pattern "0, 6, 8, 14, 18, 20, 24, 26" --tablelimit 100000
Tuple Digits: 10
Primorial Number: 5
Primorial Offset: 88793
Constellation Pattern: 0, 6, 8, 14, 18, 20, 24, 26
Prime Table Limit: 100000
Candidates of the form: 2310 * f + 88793 + 8441465340
Sieving...
Sieve Size: 100 MB
Primality Testing...
Found 88 primes, with 58014618 primality tests, eliminated 42085382
```



