# primerug
A prime k-tuple finder based on the rug Rust crate. The code is heavily inspired by Pttn's [RieMiner](https://github.com/Pttn/rieMiner) if not an attempt to make a working Rust port.


# Build
```bash
sudo apt install cargo
cargo build --release
```

# Run
```bash
makis@xps13:~/Repositories/primerug$ ./target/release/primerug 
Searching for Tuples with >= 8 digits
f: 0
primorial: 30
t_prime: 10000020
Candidates of the form: p_m * f + o + T2
Candidates of the form: 30 * f + 97 + 10000020
Sieving...
Primality Testing...
Found 142 primes, with 167267 primality tests, eliminated 9832733
```



