use alloc::boxed::Box;
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Dir {
    parent: Option<Box<Dir>>,
    name: String,
    addr: u32,
    size: u32,
    entry_index: u32,
}


