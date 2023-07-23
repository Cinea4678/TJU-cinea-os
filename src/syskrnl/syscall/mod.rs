use serde::{Deserialize, Serialize};
use x86_64::instructions::interrupts;

use cinea_os_sysapi::call::*;

use cinea_os_sysapi::ExitCode;

/// 系统调用
///
/// 2023/7/11，怀着激动的心情，创建这个mod
///

mod service;

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
            NO_SCHE => {
                service::stop_schedule();
                0
            }
            CON_SCHE => {
                service::restart_schedule();
                0
            }
            TEST_SERDE => {
                service::test_serde(arg1)
            }
            _ => panic!("unknown syscall id: {}", syscall_id),
        }
    })
}


#[cfg(test)]
mod test {
    use alloc::vec::Vec;
    use core::slice;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct TestUse {
        pub a: usize,
        pub b: u16,
    }

    #[test_case]
    fn test_serde() {
        // 模拟在调用传递过程中数据的反序列化
        let obj = TestUse { a: 100, b: 20 };
        let v = postcard::to_allocvec(&obj).unwrap();

        let addr = v.into_raw_parts();
        let info = [addr.0 as usize, addr.1, addr.2];
        let info_addr = info.as_ptr() as usize;

        let info2 = unsafe { slice::from_raw_parts(info_addr as *const usize, 3) };
        let v2 = unsafe { Vec::from_raw_parts(info2[0] as *mut u8, info2[1], info2[2]) };
        let obj2: TestUse = postcard::from_bytes(v2.as_slice()).unwrap();
        assert_eq!(obj, obj2);
        println!("[ok]  System Call test_serde")
    }
}