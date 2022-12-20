use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

struct FileInfo {
    size: i32,
    name: String,
}

struct Folder {
    parent: Option<usize>,
    name: String,
}

struct FolderLink {
    parent: usize,
    child: usize,
}

struct FileLink {
    parent: usize,
    child: usize,
}

struct PromptState {
    current_position: usize,
    folders: Vec<Folder>,
    files: Vec<FileInfo>,
    folder_links: Vec<FolderLink>,
    file_links: Vec<FileLink>,
}

impl PromptState {
    fn new() -> Self {
        let root = Folder {
            name: String::from("/"),
            parent: None,
        };
        Self {
            current_position: 0,
            folders: Vec::from([root]),
            files: Vec::new(),
            folder_links: Vec::new(),
            file_links: Vec::new(),
        }
    }

    fn move_to_root(&mut self) {
        self.current_position = 0;
    }

    fn move_to_parent(&mut self) {
        self.current_position = self
            .folder_links
            .iter()
            .find(|l| l.child == self.current_position)
            .unwrap()
            .parent
    }

    fn move_to_dir(&mut self, name: &str) {
        self.current_position = self
            .folder_links
            .iter()
            .find(|l| l.parent == self.current_position && self.folders[l.child].name == name)
            .unwrap()
            .child
    }

    fn register_dir(&mut self, name: String) {
        let id = self.folders.len();
        let folder = Folder {
            name,
            parent: Some(self.current_position),
        };
        self.folders.push(folder);
        self.folder_links.push(FolderLink {
            parent: self.current_position,
            child: id,
        });
    }

    fn register_file(&mut self, name: String, size: i32) {
        let id = self.files.len();
        let file = FileInfo { name, size };
        self.files.push(file);
        self.file_links.push(FileLink {
            parent: self.current_position,
            child: id,
        });
    }

    fn dir_size(&self, folder: usize) -> i32 {
        let direct_files_size: i32 = self
            .file_links
            .iter()
            .filter(|l| l.parent == folder)
            .map(|l| self.files[l.child].size)
            .sum();
        let subfolder_size: i32 = self
            .folder_links
            .iter()
            .filter(|l| l.parent == folder)
            .map(|l| self.dir_size(l.child))
            .sum();
        return direct_files_size + subfolder_size;
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let mut state = PromptState::new();
    let cd_regex = Regex::new(r"^\$ cd ([a-zA-Z/\.]+)$").unwrap();
    let ls_regex = Regex::new(r"^\$ ls$").unwrap();
    let ls_output_regex = Regex::new(r"(dir|\d+) ([a-zA-Z\.]+)").unwrap();
    for line in lines {
        let content = line.unwrap();
        match cd_regex.captures(&content) {
            Some(capt) => {
                let dest = capt.get(1).unwrap();
                match dest.as_str() {
                    "/" => state.move_to_root(),
                    ".." => state.move_to_parent(),
                    dest => state.move_to_dir(dest),
                }
                continue;
            }
            None => {}
        }
        match ls_regex.captures(&content) {
            Some(_) => continue,
            None => {}
        }
        match ls_output_regex.captures(&content) {
            Some(capt) => {
                let first_part = capt.get(1).unwrap().as_str();
                let second_part = capt.get(2).unwrap().as_str();
                match (first_part, second_part) {
                    ("dir", name) => {
                        state.register_dir(String::from(name));
                    }
                    (size, name) => {
                        state.register_file(String::from(name), i32::from_str(size).unwrap())
                    }
                }
                continue;
            }
            None => {}
        }
    }
    let available_space = 70000000 - state.dir_size(0);
    let minimum_to_delete = 30000000 - available_space;
    let minimum_folder_to_delete = state
        .folders
        .iter()
        .enumerate()
        .map(|(i, _)| state.dir_size(i))
        .filter(|s| *s > minimum_to_delete)
        .min()
        .unwrap();
    println!(
        "The smallest directory we cna delete has a size of {}",
        minimum_folder_to_delete
    );
    Ok(())
}
