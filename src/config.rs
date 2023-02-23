use rug::Integer;

#[derive(Clone)]
pub struct Config {
    pub d: u32,
    pub constellation_pattern: Vec<u64>,
    pub m: u64,
    pub o: u64,
    pub prime_table_limit: u64,
    pub threads: usize,
    pub primorial: Integer,
}

impl Config {
    pub fn new(
        d: u32,
        constellation_pattern: String,
        m: u64,
        o: u64,
        prime_table_limit: u64,
        threads: usize,
        primorial: Integer,
    ) -> Config {
        let v = Self::get_pattern_vector(constellation_pattern);

        Config {
            d,
            constellation_pattern: v,
            m,
            o,
            prime_table_limit,
            threads,
            primorial,
        }
    }

    fn get_pattern_vector(offsets: String) -> Vec<u64> {
        let offsets: String = offsets.chars().filter(|c| !c.is_whitespace()).collect();

        let str_pattern = offsets.split(",");

        let mut pattern_vector: Vec<u64> = Vec::new();

        for o in str_pattern {
            pattern_vector.push(o.parse::<u64>().unwrap());
        }

        pattern_vector
    }
}
