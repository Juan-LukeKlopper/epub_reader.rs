use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the epub file
    #[arg(short, long)]
    path: String,

    /// words per minute used to calculate estimated reading time
    /// 238 is the Adult Average Reading Speed so is a sensible default
    #[arg(short, long, default_value_t = 238)]
    word_count: u16,
}

fn main() {
    let args = Args::parse();

    println!("args = {:?}", args);
}
