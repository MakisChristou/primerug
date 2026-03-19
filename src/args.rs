use clap::Parser;

/// A prime k-tuple finder based on the rug crate.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Number of decimal digits in the target
    #[arg(short, long)]
    pub digits: u32,

    /// Primorial number (0 = auto-select based on digit count)
    #[arg(short, long, default_value_t = 0)]
    pub m: u64,

    /// Primorial offset (0 = auto-select based on pattern)
    #[arg(short, long, default_value_t = 0)]
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

    /// Number of sieve iterations per target (amortizes presieve cost)
    #[arg(short = 'i', long = "sieve-iterations", default_value_t = 10)]
    pub sieve_iterations: u32,

    /// Number of dedicated sieve workers (0 = monolithic mode, >0 = sieve-worker split)
    #[arg(long = "sieve-workers", default_value_t = 0)]
    pub sieve_workers: u32,

    /// Path to GPU service Unix socket (empty = CPU-only mode)
    #[arg(long = "gpu-socket", default_value = "")]
    pub gpu_socket: String,

    /// Number of candidates per GPU batch
    #[arg(long = "gpu-batch-size", default_value_t = 32768)]
    pub gpu_batch_size: u32,
}
