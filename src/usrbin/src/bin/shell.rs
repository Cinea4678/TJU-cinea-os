#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use alloc::vec::Vec;
use cinea_os_sysapi::stdin::get_line_string;
use cinea_os_sysapi::{allocator, entry_point};
use cinea_os_userspace::print;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

//const VERSION:&str = "v0.1.0";

enum ResolveError {
    BrokenQuote,
}

fn resolve_command(command: &str) -> Result<Vec<String>, ResolveError> {
    let mut result: Vec<String> = Vec::new();
    let mut str = String::new();
    let mut saved = false;
    let mut reading_quote = usize::MAX;
    let mut reading_double_quote = usize::MAX;
    let mut start = 0usize;

    let mut save = |pos: usize| {
        if saved == false {
            let mut new_str = String::from(&command[start..pos]);
            result.push(new_str);
            saved = true;
        }
    };

    let chars: Vec<_> = command.chars().collect();
    for i in 0..command.len() {
        if chars[i] == '\n' {
            // 尝试结束
        }
    }

    Ok(Vec::new())
}

fn main(_args: &[&str]) {
    let nowdir = String::from("~");

    loop {
        print!("{} $ ", nowdir.as_str());

        let cmd = get_line_string(false);

        loop {}
    }
}
