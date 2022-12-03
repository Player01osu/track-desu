use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    env,
    io,
    path::{Path, PathBuf},
};
lazy_static! {
    static ref GIT_DIR: PathBuf = PathBuf::from("/home/bruh/Documents/.tmp/");
}

const HELP: &'static str = r#"usage: tracker-desu

Track dotfiles in a git directory with a single command."#;
const VERSION: &str = "Tracker-desu: 0.0.0";

fn track_file<T: AsRef<Path>>(path: &T) -> Result<(), io::Error> {
    // Copy the file from one directory to the git dir.
    // Compare checksum between two files.
    // Delete file from current path if checksums match.
    // Delete file from git dir and err if checksums do not
    // match.
    // Create a symlink from the git dir to the
    // given file.
    let to_file = format!("{}/{}", GIT_DIR.display(), path.as_ref().display());
    std::fs::copy(path, to_file)?;

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum Argument {
    Help,
    Version,
    Verbose,
    Copy,
    Symlink,
    Recursive,
    DryRun,
    NoConfirm,
    // TODO
    GitDir,
    File(String),
}

use Argument::*;

fn short_form_args(args: &str) -> Vec<Result<Argument>> {
    args.chars()
        .map(|c| match c {
            'r' => Ok(Recursive),
            'h' => Ok(Help),
            'n' => Ok(DryRun),
            'c' => Ok(Copy),
            's' => Ok(Symlink),
            'C' => Ok(NoConfirm),
            'd' => Ok(GitDir),
            'v' => Ok(Verbose),
            'V' => Ok(Version),
            _ => return Err(anyhow!("Invalid argument: {}", c)),
        })
        .collect::<Vec<Result<Argument>>>()
}

use anyhow::{anyhow, Result};

fn match_args(arg: &str) -> Vec<Result<Argument>> {
    match arg {
        "--recursive" => {
            vec![Ok(Recursive)]
        }
        "--help" => {
            vec![Ok(Help)]
        }
        "--dry-run" => {
            vec![Ok(DryRun)]
        }
        "--copy" => {
            vec![Ok(Copy)]
        }
        "--symlink" => {
            vec![Ok(Symlink)]
        }
        "--no-confirm" => {
            vec![Ok(NoConfirm)]
        }
        "--git-dir" => {
            vec![Ok(GitDir)]
        }
        "--verbose" => {
            vec![Ok(Verbose)]
        }
        "--version" => {
            vec![Ok(Version)]
        }
        s => {
            if &s[0..1] == "-" {
                return short_form_args(&s[1..]);
            }

            if &s[0..2] == "--" {
                return vec![Err(anyhow!("Invalid argument: {}", s))];
            }

            vec![Ok(File(s.to_owned()))]
        }
    }
}

fn execute_args(args: &[Argument]) -> Result<()> {
    for arg in args {
        match arg {
            Help => {
                eprintln!("{HELP}");
                return Ok(());
            }
            Version => {
                eprintln!("{VERSION}");
                return Ok(());
            }
            GitDir => {
                unimplemented!()
            }
            File(ref path) => {
                track_file(path)?;
            }
            _ => unimplemented!(),
        }
    }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<()> {
    let mut args = args
        .iter()
        .dedup()
        .flat_map(|arg| match_args(arg))
        .collect::<Result<Vec<Argument>>>()?;
    args.sort();

    execute_args(&args)?;

    dbg!(args);
    Ok(())
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    match args.len() {
        1 => {
            eprintln!("{HELP}");
            return Ok(());
        }
        n => {
            parse_args(&args[1..])?;
        }
    }
    Ok(())
}
