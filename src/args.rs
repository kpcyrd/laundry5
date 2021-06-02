use std::net::SocketAddr;
use std::path::PathBuf;
// use structopt::clap::{AppSettings, Shell};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Only show warnings
    #[structopt(short, long, global = true)]
    pub quiet: bool,
    /// More verbose logs
    #[structopt(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,
    /// The address to bind to
    #[structopt(short = "B", long, default_value = "127.0.0.1:1080")]
    pub bind: SocketAddr,
    /// The path to the proxy list to use
    #[structopt(short = "L", long)]
    pub proxy_list: PathBuf,
}
