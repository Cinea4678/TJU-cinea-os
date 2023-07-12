pub mod syscall;
pub mod proc;

#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            $crate::sysapi::syscall::log(b"An exception occured!\n");
            loop {}
        }

        #[export_name = "_start"]
        pub unsafe extern "sysv64" fn __impl_start(args_ptr: u64, args_len: usize) {
            let args = core::slice::from_raw_parts(args_ptr as *const _, args_len);
            let f: fn(&[&str]) = $path;
            f(args);
            $crate::sysapi::syscall::exit($crate::sysapi::proc::ExitCode::Success);
        }
    };
}

