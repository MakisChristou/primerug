use clap::Parser;

/// A prime k-tuple finder based on the rug crate.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Number of decimal digits in the target
    #[arg(short, long)]
    pub digits: u32,

    /// Primorial number
    #[arg(short, long, default_value_t = 3)]
    pub m: u64,

    /// Primorial offset
    #[arg(short, long, default_value_t = 97)]
    pub o: u64,

    /// Constellation pattern (comma-separated offsets)
    #[arg(short, long, default_value = "0, 4, 6, 10, 12, 16")]
    pub pattern: String,

    /// Upper limit for prime table generation
    #[arg(short = 'l', long = "tablelimit", default_value_t = 100_000)]
    pub table_limit: u64,

    /// Stats printing interval in seconds
    #[arg(short = 's', long = "interval", default_value_t = 5)]
    pub stats_interval: u64,

    /// Number of worker threads
    #[arg(short, long, default_value_t = 1)]
    pub threads: usize,
}
