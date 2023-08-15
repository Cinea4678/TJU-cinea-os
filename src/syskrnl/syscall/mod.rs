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
            EXIT => service::exit(ExitCode::from(arg1)),
            SPAWN => service::spawn(arg1, arg2, arg3, arg4) as usize,
            INFO => service::info(arg1),
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
                service::panic(arg1)
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
            LIST => {
                service::list(arg1)
            }
            OPEN => {
                service::open(arg1)
            }
            WRITE_ALL => {
                service::write_all(arg1)
            }
            READ => {
                service::read(arg1)
            }
            WRITE_PATH => {
                service::write_path(arg1)
            }
            READ_PATH => {
                service::read_path(arg1)
            }
            SPAWN_FROM_PATH => {
                service::spawn_from_path(arg1)
            }
            CREATE_WINDOW => {
                service::create_window(arg1)
            }
            DISPLAY_FONT_STRING => {
                service::display_font_string(arg1)
            }
            LOAD_FONT => {
                service::load_font(arg1)
            }
            DESTROY_WINDOW => {
                service::destroy_window()
            }
            REGISTER_TIMER => {
                service::register_timer(arg1)
            }
            GUI_SUBSCRIBE_TIME_UPDATE => {
                service::gui_time_update_register()
            }
            READ_TIME => {
                service::read_time()
            }
            GUI_SUBSCRIBE_KEYBOARD => {
                service::gui_time_update_register()
            }
            _ => panic!("unknown syscall id: {}", syscall_id),
        }
    })
}

#[macro_export]
macro_rules! syscall_serialized_ret {
    ($($arg:tt)*) => {
        if $crate::syskrnl::proc::id()==0 {
            cinea_os_sysapi::call::syscall_serialized($($arg)*)
        }else{
            let allocator = $crate::syskrnl::proc::heap_allocator().clone();
            cinea_os_sysapi::call::syscall_serialized_for_userspace($($arg)*, |x| unsafe { allocator.lock().alloc(x) })
        }
    };
}

#[macro_export]
macro_rules! syscall_deserialize {
    ($ptr:expr) => {{
        use cinea_os_sysapi::call::{syscall_deserialized, syscall_deserialized_prepare};
        let vec_data = syscall_deserialized_prepare($ptr);
        syscall_deserialized(&vec_data).unwrap()
    }};
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