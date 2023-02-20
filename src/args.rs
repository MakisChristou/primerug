use clap::Parser;

/// A prime k-tuple finder based on the rug Rust crate.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Size of the tuple in decimal digits
   #[arg(short, long)]
   pub digits: u32,

   /// Primorial number
   #[arg(short, long, default_value_t = 3)]
   pub m: u64,

   /// Primorial offset
   #[arg(short, long, default_value_t = 97)]
   pub o: u64,

   /// Desired pattern
   #[arg(short, long, default_value_t = String::from("0, 4, 6, 10, 12, 16"))]
   pub pattern: String,

   /// Desired pattern
   #[arg(short, long, default_value_t = 100_000)]
   pub tablelimit: u64,

   /// Stats interval
   #[arg(short, long, default_value_t = 5)]
   pub interval: usize,

   /// Threads
   #[arg(short, long, default_value_t = 1)]
   pub threads: usize,
}