mod args;
mod file;

use anyhow::Result;
use args::{Opt, parse_args};
use file::{path_check, copy_track, symlink_track};
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    path::PathBuf,
};

lazy_static! {
    static ref GIT_DIR: PathBuf = PathBuf::from("/home/bruh/Documents/.tmp/")
        .canonicalize()
        .unwrap();
}

const HELP: &'static str = r#"usage: tracker-desu

Track dotfiles in a git directory with a single command."#;
const VERSION: &str = "Tracker-desu: 0.0.0";

fn track(opts: &HashSet<Opt>, files: Vec<&String>) -> Result<()> {
    if opts.contains(&Opt::Symlink) {
        for path in files {
            symlink_track(path, opts.contains(&Opt::Recursive))?;
        }
    } else {
        for path in files {
            path_check(path)?;
            copy_track(path, opts.contains(&Opt::Recursive))?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    match args.len() {
        1 => {
            eprintln!("{HELP}");
            return Ok(());
        }
        _ => {
            parse_args(&args[1..])?;
        }
    }
    Ok(())
}
