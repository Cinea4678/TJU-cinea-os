pub mod syscall;
pub mod proc;
pub mod allocator;
pub mod call;

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

#[macro_export]
macro_rules! syscall {
    ($n:expr) => (
        $crate::sysapi::syscall::syscall0(
            $n as usize));
    ($n:expr, $a1:expr) => (
        $crate::sysapi::syscall::syscall1(
            $n as usize, $a1 as usize));
    ($n:expr, $a1:expr, $a2:expr) => (
        $crate::sysapi::syscall::syscall2(
            $n as usize, $a1 as usize, $a2 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => (
        $crate::sysapi::syscall::syscall3(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => (
        $crate::sysapi::syscall::syscall4(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize, $a4 as usize));
}
