use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "tim2-viewer", about = "View tim2 files from a given directory", version = env!("CARGO_PKG_VERSION"))]
pub struct Opts {
    #[structopt(parse(from_os_str))]
    pub directory: PathBuf,
}
