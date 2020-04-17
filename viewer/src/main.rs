use anyhow::{bail, Error};
use iced::{window, Application, Settings};
use structopt::StructOpt;

mod cli;
mod viewer;

pub fn main() -> Result<(), Error> {
    let opts = cli::Opts::from_args();

    if !opts.directory.is_dir() {
        bail!("<directory> must be a valid directory");
    }

    viewer::Viewer::run(Settings {
        flags: viewer::Flags {
            directory: opts.directory,
        },
        window: window::Settings {
            //size: (512, 512),
            ..Default::default()
        },
        ..Default::default()
    });

    Ok(())
}
