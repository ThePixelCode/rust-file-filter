use std::{env::{Args, current_dir}, path::PathBuf, process::exit};

use enums::InConflictDo;
use filterbyhash::FilterByHash;
use structs::Config;
use traits::OperationHandler;

mod filterbyhash;
mod traits;
mod structs;
mod enums;

pub fn print_error_and_gracefully_exit(error: &str) -> ! {
    println!("An Error Ocurred
    The Error was: {}", error);
    exit(1);
}

fn check_absolute_path(path: String) -> Result<String, &'static str> {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        return Ok(format!("{}", path_buf.display()));
    }
    match path_buf.canonicalize() {
        Ok(path) => Ok(format!("{}", path.display())),
        Err(_) => Err("Unable to resolve path"),
    }
}

fn get_config(args: Args) -> Result<Config, &'static str> {
    let mut folder = None;
    let mut in_conflict_do = InConflictDo::Delete;
    let mut iter = args.into_iter();
    loop {
        let arg = match iter.next() {
            Some(arg) => arg,
            None => break,
        };
        if arg.starts_with("-") {
            match arg.as_str() {
                "-f" | "--folder" => {
                    folder = match iter.next() {
                        Some(folder) => Some(folder),
                        None => return Err("No folder specified"),
                    }
                }
                "-d" | "--delete" => in_conflict_do = InConflictDo::Delete,
                "-a" | "--ask" => in_conflict_do = InConflictDo::Ask,
                "-i" | "--inform" => in_conflict_do = InConflictDo::Inform,
                "-m" | "--move" => {
                    let move_to = match iter.next() {
                        Some(folder) => match check_absolute_path(folder) {
                            Ok(path) => path,
                            Err(e) => return Err(e),
                        },
                        None => return Err("No folder specified"),
                    };
                    in_conflict_do = InConflictDo::Move(move_to);
                }
                _ => return Err("Invalid argument"),
            }
            continue;
        }
    }
    let folder = match folder.map(|path| check_absolute_path(path)) {
        Some(result) => match result {
            Ok(path) => path,
            Err(e) => return Err(e),
        },
        None => match current_dir() {
            Ok(path) => format!("{}", path.display()),
            Err(_) => return Err("Can't see the current directory"),
        },
    };
    return Ok(Config {
        folder,
        in_conflict_do,
    });
}

pub fn get_operation_handler(args: Args) -> Result<Box<dyn OperationHandler>, &'static str> {
    let config = match get_config(args) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };
    return Ok(Box::new(FilterByHash {
        folder: config.folder,
        in_conflict_do: config.in_conflict_do,
        hash_list: Vec::new(),
    }));
}