use std::path::{Path, PathBuf};

#[derive(Debug,Clone,PartialEq, Eq, PartialOrd, Ord)]
pub struct MySerialPort {
    pub path : PathBuf
}

impl std::fmt::Display for MySerialPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}",&self.path.to_string_lossy()))
    }
}

impl MySerialPort {
    pub fn new(path : &Path) -> Self {
        Self {
            path : path.to_path_buf()
        }
    }
}