use std::{env::{Args, current_dir}, path::{PathBuf, Path}, process::exit, fs::{read_dir, File, remove_file, rename}, io::Read};

use ring::digest::{Context, SHA256, Digest};

use hex;

pub fn print_error_and_gracefully_exit(error: &str) -> ! {
    println!("An Error Ocurred
    The Error was: {}", error);
    exit(1);
}

pub trait OperationHandler {
    fn run(&mut self) -> Result<(), &'static str>;
}

enum InConflictDo {
    Delete,
    Ask,
    Inform,
    Move(String),
}

struct Config {
    folder: String,
    in_conflict_do: InConflictDo,
}


struct FilterByHash {
    folder: String,
    in_conflict_do: InConflictDo,
    hash_list: Vec<Digest>,
}

impl OperationHandler for FilterByHash {
    fn run(&mut self) -> Result<(), &'static str> {
        let files: Vec<_> = match read_dir(&self.folder) {
            Ok(entries) => {
                entries.filter_map(|entry| {
                    if let Ok(entry) = entry {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                let file_name = entry.file_name();
                                return Some(PathBuf::from(&self.folder).join(file_name))
                            }
                        }
                    }
                    None
                }).collect()
            },
            Err(_) => return Err("Unable to read folder"),
        };
        for file_path in files {
            let file = match File::open(&file_path) {
                Ok(file) => Some(file),
                Err(_) => None,
            };
            if let Some(mut file) = file {
                let mut buffer = Vec::new();
                match file.read_to_end(&mut buffer) {
                    Ok(_) => (),
                    Err(_) => continue,
                }
                let mut context = Context::new(&SHA256);
                context.update(&buffer);
                let digest = context.finish();
                match self.hash_list.binary_search_by(|probe| {
                    probe.as_ref().cmp(digest.as_ref())
                }) {
                    Ok(_) => {
                        match &self.in_conflict_do {
                            InConflictDo::Delete => {
                                match remove_file(&file_path) {
                                    Ok(_) => {
                                        println!("File {} deleted", &file_path.display());
                                    },
                                    Err(_) => return Err("Unable to delete file"),
                                }
                            },
                            InConflictDo::Ask => todo!(),
                            InConflictDo::Inform => {
                                println!("File {} is repeated, hash: {}", &file_path.display(), hex::encode(digest));
                            },
                            InConflictDo::Move(folder) => {
                                let file_name = file_path.file_name().unwrap();
                                let destination = Path::new(folder.as_str()).join(file_name);
                                match rename(&file_path, &destination) {
                                    Ok(_) => {
                                        println!("File {} moved to {}", &file_path.display(), &destination.display());
                                    },
                                    Err(_) => return Err("Unable to move file"),
                                }
                            },
                        }
                    },
                    Err(index) => {
                        self.hash_list.insert(index, digest);
                    },
                }
            }
        }
        return Ok(());
    }
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