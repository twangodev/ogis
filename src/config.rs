use clap::Parser;

#[derive(Parser)]
#[command(name = "ogis")]
#[command(version)]
#[command(about = "Open Graph Image Service - Generate OG images dynamically")]
pub struct Config {
    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:3000", env = "OGIS_ADDR")]
    pub addr: String,
}

impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}