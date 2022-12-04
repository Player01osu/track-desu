use std::{path::Path, ffi::OsStr, fmt::Display, io};

use crate::GIT_DIR;

pub fn symlink_track<T: AsRef<Path> + AsRef<OsStr> + Display>(
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

pub fn copy_track<T: AsRef<Path> + AsRef<OsStr> + Display>(
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


pub fn path_check<T: AsRef<Path> + AsRef<OsStr>>(path: &T) -> Result<(), io::Error>{
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

