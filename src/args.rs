
use std::collections::HashSet;

use anyhow::{Result, anyhow};
use itertools::Itertools;
use crate::{HELP, VERSION};
use crate::track;
use Argument::*;

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum Argument {
    Help,
    Version,
    Verbose,
    DryRun,
    Copy,
    Symlink,
    Recursive,
    NoConfirm,
    // TODO No Checksum arg
    GitDir, // TODO
    File(String),
}


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
            _ => {
                return Err(anyhow!("Invalid argument: {}", c));
            }
        })
        .collect::<Vec<Result<Argument>>>()
}

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

            if s.len() > 2 && &s[0..2] == "--" {
                return vec![Err(anyhow!("Invalid argument: {}", s))];
            }

            vec![Ok(File(s.to_owned()))]
        }
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Opt {
    NoConfirm,
    Symlink,
    Copy,
    DryRun,
    Verbose,
    Recursive,
}


fn execute_args(args: Vec<Argument>) -> Result<()> {
    let mut opts = HashSet::new();
    let mut files = vec![];

    for arg in args.iter() {
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
            Verbose => {
                opts.insert(Opt::Verbose);
            }
            Copy => {
                if opts.contains(&Opt::Symlink) {
                    return Err(anyhow!("Copy conflicts with Symlink option"));
                }
                opts.insert(Opt::Copy);
            }
            Symlink => {
                if opts.contains(&Opt::Copy) {
                    return Err(anyhow!("Copy conflicts with Symlink option"));
                }
                opts.insert(Opt::Symlink);
            }
            DryRun => {
                opts.insert(Opt::DryRun);
            }
            Recursive => {
                opts.insert(Opt::Recursive);
            }
            NoConfirm => {
                opts.insert(Opt::NoConfirm);
            }
            File(ref path) => {
                files.push(path);
            }
        }
    }

    if files.is_empty() {
        return Err(anyhow!("missing file operand"));
    }

    track(&opts, files)?;

    Ok(())
}

pub fn parse_args(args: &[String]) -> Result<()> {
    let mut args = args
        .iter()
        .dedup()
        .flat_map(|arg| match_args(arg))
        .collect::<Result<Vec<Argument>>>()?;
    args.sort_by(|p, o| o.cmp(p));

    execute_args(args)?;

    Ok(())
}
