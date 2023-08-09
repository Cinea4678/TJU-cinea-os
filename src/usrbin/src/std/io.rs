use core::convert::Infallible;
use alloc::string::String;
use ufmt::uWrite;

pub struct StdWriter;

impl uWrite for StdWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        cinea_os_sysapi::syscall::log(s.as_bytes());
        Ok(())
    }
}

pub struct StringWriter{
    value: String
}

impl uWrite for StringWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.value += s;
        Ok(())
    }
}

impl StringWriter {
    pub fn new()->Self{
        Self { value: String::new() }
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        let mut std_writer = $crate::std::StdWriter;
        ufmt::uwrite!(std_writer, $($arg)*).unwrap();
    })
}

#[macro_export]
macro_rules! debug {
    ($exp:expr) => ({
        $crate::print!("{}", serde_json::to_string(&$exp).unwrap().as_str());
    })
}

#[macro_export]
macro_rules! debugln {
    ($exp:expr) => ({
        $crate::print!("{}\n", serde_json::to_string(&$exp).unwrap().as_str());
    })
}

// #[macro_export]
// macro_rules! println {
//     () => ($crate::print!("\n"));
//     ($fmt:expr $(, $($arg:tt)+)?) => {
//         {
//             let mut std_writer = crate::std::StdWriter;
//             ufmt::uwrite!(std_writer, concat!($fmt, "\n") $(, $($arg)+)?).unwrap();
//         }
//     };
// }

#[macro_export]
macro_rules! printf {
    ($($arg:tt)*) => ({
        let args = alloc::vec![$($arg)*];
        cinea_os_sysapi::syscall_with_serialize!(cinea_os_sysapi::call::PRINTF, &args);
    })
}

