use anyhow::{anyhow, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fmt::Display,
    io,
    mem::size_of_val,
    path::{Path, PathBuf},
};
lazy_static! {
    static ref GIT_DIR: PathBuf = PathBuf::from("/home/bruh/Documents/.tmp/")
        .canonicalize()
        .unwrap();
}

const HELP: &'static str = r#"usage: tracker-desu

Track dotfiles in a git directory with a single command."#;
const VERSION: &str = "Tracker-desu: 0.0.0";

fn path_check<T: AsRef<Path> + AsRef<OsStr>>(path: &T) -> Result<(), io::Error>{
    let path = Path::new(path);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("cannot stat '{}': No such file or directory", path.to_string_lossy()),
        ));
    }
    Ok(())
}

fn recursive_copy<T: AsRef<Path> + AsRef<OsStr>>(path: &T, to_file: &str) -> Result<(), io::Error> {
    path_check(path)?;
    let path = Path::new(path);
    fn _recurse(path: &Path, to_file: &str) -> Result<(), io::Error> {
        if path.is_file() {
            std::fs::copy(path, &to_file)?;
        } else if path.is_symlink() {
            std::os::unix::fs::symlink(std::fs::canonicalize(path)?, to_file)?;
        } else {
            std::fs::create_dir(&to_file)?;
            for path in path.read_dir()? {
                let path = path?.path();
                let to_file = format!("{to_file}/{}", &path.file_name().unwrap().to_string_lossy());
                _recurse(&path, &to_file)?;
            }
        }
        Ok(())
    }

    _recurse(path, to_file)?;

    Ok(())
}

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

fn symlink_track<T: AsRef<Path> + AsRef<OsStr> + Display>(
    path: &T,
    recursive: bool,
) -> Result<(), io::Error> {
    let to_file = format!("{}/{}", GIT_DIR.display(), path);
    if recursive {
        recursive_copy(path, &to_file)?;
        std::fs::remove_file(path).unwrap_or_else(|_| std::fs::remove_dir_all(path).unwrap());
    } else {
        std::fs::copy(path, &to_file)?;
        std::fs::remove_file(path).unwrap();
    }

    // TODO Check hash before removing
    std::os::unix::fs::symlink(&to_file, path).unwrap();

    Ok(())
}

fn copy_track<T: AsRef<Path> + AsRef<OsStr> + Display>(
    path: &T,
    recursive: bool,
) -> Result<(), io::Error> {
    // Copy the file from one directory to the git dir.
    // Compare checksum between two files.
    // Delete file from current path if checksums match.
    // Delete file from git dir and err if checksums do not
    // match.
    // Create a symlink from the git dir to the
    // given file.
    let to_file = format!("{}/{}", GIT_DIR.display(), &path);
    if recursive {
        recursive_copy(path, &to_file)?;
    } else {
        std::fs::copy(path, &to_file)?;
    }

    Ok(())
}

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
            _ => return Err(anyhow!("Invalid argument: {}", c)),
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
enum Opt {
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

    track(&opts, files)?;

    Ok(())
}

fn parse_args(args: &[String]) -> Result<()> {
    let mut args = args
        .iter()
        .dedup()
        .flat_map(|arg| match_args(arg))
        .collect::<Result<Vec<Argument>>>()?;
    args.sort_by(|p, o| o.cmp(p));

    execute_args(args)?;

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
