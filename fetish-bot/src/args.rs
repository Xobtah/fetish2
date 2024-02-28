use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "db_me")]
    pub database_directory: String,
}
