use std::{fs::{read_dir, File, remove_file, rename}, path::{PathBuf, Path}, io::{Read, BufRead, stdin}};

use ring::digest::{Digest, Context, SHA256};

use crate::{enums::InConflictDo, traits::OperationHandler, check_absolute_path};

pub struct FilterByHash {
    pub folder: String,
    pub in_conflict_do: InConflictDo,
    pub hash_list: Vec<Digest>,
}

impl FilterByHash {
    fn delete_file(&self, file_path: &PathBuf) -> Result<(), &'static str> {
        match remove_file(&file_path) {
            Ok(_) => {
                println!("File {} deleted", &file_path.display());
                Ok(())
            },
            Err(_) => return Err("Unable to delete file"),
        }
    }

    fn move_file(&self, file_path: &&PathBuf, folder: &str) -> Result<(), &'static str> {
        let file_name = file_path.file_name().unwrap();
        let destination = Path::new(folder).join(file_name);
        match rename(&file_path, &destination) {
            Ok(_) => Ok({
                println!("File {} moved to {}", &file_path.display(), &destination.display());
            }),
            Err(_) => return Err("Unable to move file"),
        }
    }

    fn ask_for_conflict_file(&self, file_path: &PathBuf) -> Result<(), &'static str> {
        println!("File {} is repeated, what to do with it? (delete/move/ignore)", &file_path.display());
        let mut input = String::new();
        let mut stdin = stdin().lock();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                match input.trim() {
                    "delete" => {
                        match self.delete_file(&file_path) {
                            Ok(_) => Ok(()),
                            Err(e) => return Err(e),
                        }
                    },
                    "move" => {
                        println!("Enter destination folder:");
                        let mut input = String::new();
                        match stdin.read_line(&mut input) {
                            Ok(_) => {
                                match check_absolute_path(input).and_then(|path| {
                                                                    self.move_file(&file_path, &path)
                                                                }) {
                                    Ok(_) => {()},
                                    Err(e) => return Err(e),
                                }
                            },
                            Err(_) => return Err("Unable to read input"),
                        }
                        Ok(())
                    },
                    "ignore" => Ok(()),
                    _ => return Err("Wrong input"),
                }
            },
            Err(_) => {
                return Err("Unable to read input");
            },
        }
    }

    fn resolve_conflict(&self, file_path: &PathBuf, digest: &Digest) -> Result<(), &'static str> {
        match &self.in_conflict_do {
            InConflictDo::Delete => {
                match self.delete_file(&file_path) {
                    Ok(_) => {()},
                    Err(e) => return Err(e),
                }
            },
            InConflictDo::Ask => match self.ask_for_conflict_file(&file_path) {
                Ok(_) => {()},
                Err(e) => return Err(e),
            },
            InConflictDo::Inform => {
                println!("File {} is repeated, hash: {}", &file_path.display(), hex::encode(digest));
            },
            InConflictDo::Move(folder) => {
                match self.move_file(&file_path, folder) {
                    Ok(_) => {()},
                    Err(e) => return Err(e),
                }
            },
        }
        return Ok(());
    }
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
                    Ok(_) => {match self.resolve_conflict(&file_path, &digest) {
                        Ok(_) => {()},
                        Err(e) => return Err(e),
                    }},
                    Err(index) => {
                        self.hash_list.insert(index, digest);
                    },
                }
            }
        }
        return Ok(());
    }
}
