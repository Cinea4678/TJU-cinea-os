#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Add;

use cinea_os_sysapi::{allocator, entry_point};
use cinea_os_sysapi::fs::spawn_from_path;
use cinea_os_sysapi::stdin::get_line_string;
use cinea_os_sysapi::syscall::spawn;
use cinea_os_userspace::print;

use crate::ResolveError::BrokenQuote;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

//const VERSION:&str = "v0.1.0";

#[inline(always)]
fn is_blank_character(ch: char) -> bool {
    match ch {
        ' ' | '\t' | '\n' => true,
        _ => false
    }
}

enum ResolveError {
    BrokenQuote,
}

fn resolve_command(command: &str) -> Result<Vec<String>, ResolveError> {
    let mut result = Vec::<String>::new();
    let mut saved = false;
    let mut reading_quote = usize::MAX;
    let mut reading_double_quote = usize::MAX;
    let mut start = 0usize;

    let save_current_read = |pos: usize, result: &mut Vec<_>, saved: &mut bool, start: usize| {
        // print!("Save\n");
        // debugln!(pos);
        // debugln!(saved);
        // debugln!(start);
        if *saved == false {
            let new_str = String::from(&command[start..pos]);
            result.push(new_str);
            *saved = true;
        }
    };


    let chars: Vec<_> = command.chars().collect();
    for i in 0..command.len() {
        if chars[i] == '\n' {
            // 尝试结束
            save_current_read(i, &mut result, &mut saved, start);
            return Ok(result);
        } else if is_blank_character(chars[i]) && reading_quote > i && reading_double_quote > i {
            save_current_read(i, &mut result, &mut saved, start);
        } else if chars[i] == '\'' {
            // 单引号
            if reading_quote > i {
                // 初遇
                saved = false;
                reading_quote = i;
                start = i + 1;
            } else {
                // 重逢
                save_current_read(i, &mut result, &mut saved, start);
                reading_quote = usize::MAX;
            }
        } else if chars[i] == '\"' {
            // 单引号
            if reading_double_quote > i {
                // 初遇
                saved = false;
                reading_double_quote = i;
                start = i + 1;
            } else {
                // 重逢
                save_current_read(i, &mut result, &mut saved, start);
                reading_double_quote = usize::MAX;
            }
        } else {
            if saved {
                saved = false;
                start = i;
            }
        }
    }
    if saved == false {
        save_current_read(chars.len(), &mut result, &mut saved, start);
    }

    if reading_quote < usize::MAX || reading_double_quote < usize::MAX {
        return Err(BrokenQuote);
    }

    Ok(result)
}

fn main(_args: &[&str]) {
    let nowdir = String::from("~");

    loop {
        print!("{} $ ", nowdir.as_str());

        let cmd = get_line_string(false);
        match resolve_command(cmd.as_str()) {
            Err(ResolveError::BrokenQuote) => { print!("不合法的引号\n") },
            Ok(resolved) => {
                if resolved.len() == 0 {
                    continue;
                }
                print!("---Debug Message---\n");
                print!("Command: {}\n", resolved[0].as_str());
                print!("Arg Num: {}\n", resolved.len() - 1);
                print!("Args:    ");
                for i in 1..resolved.len() {
                    print!("{} ", resolved[i].as_str())
                }
                print!("\n-------------------\n");
                let exec_path = String::from("/bin/").add(resolved[0].as_str());
                if !spawn_from_path(exec_path.as_str(), resolved.as_slice()[1..].iter().cloned().collect()) {
                    print!("程序\"{}\"没有找到", resolved[0].as_str());
                }
            }
        }
    }
}
