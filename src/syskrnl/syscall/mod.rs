use x86_64::instructions::interrupts;

use call::*;

use crate::syskrnl::sysapi::ExitCode;

/// 系统调用
///
/// 2023/7/11，怀着激动的心情，创建这个mod
///

mod service;
pub mod call;

pub fn dispatcher(syscall_id: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    interrupts::without_interrupts(|| {
        match syscall_id {
            EXIT => service::exit(ExitCode::from(arg1)) as usize,
            SPAWN => service::spawn(arg1, arg2, arg3, arg4) as usize,
            READ => unimplemented!(),
            WRITE => unimplemented!(),
            OPEN => unimplemented!(),
            CLOSE => unimplemented!(),
            INFO => unimplemented!(),
            DUP => unimplemented!(),
            DELETE => unimplemented!(),
            STOP => unimplemented!(),
            SLEEP => {
                service::sleep(f64::from_bits(arg1 as u64));
                0
            }
            LOG => {
                service::log(arg1, arg2)
            }
            ALLOC => {
                service::alloc(arg1, arg2)
            }
            FREE => {
                service::free(arg1, arg2, arg3);
                0
            }
            PANIC => {
                panic!("User space program asked to panic. ACCR.");
            }
            _ => panic!("unknown syscall id: {}", syscall_id),
        }
    })
}

