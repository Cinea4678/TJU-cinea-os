use core::fmt::{Debug, Formatter};

use oem_cp::code_table::{DECODING_TABLE_CP437, ENCODING_TABLE_CP437};

pub struct Cp437Converter;

impl Debug for Cp437Converter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "TABLE_CP437_OEM_CONVERTER")
    }
}

impl fatfs::OemCpConverter for Cp437Converter {
    fn decode(&self, oem_char: u8) -> char {
        if oem_char < 128 {
            oem_char as char
        } else {
            DECODING_TABLE_CP437[oem_char as usize - 128]
        }
    }

    fn encode(&self, uni_char: char) -> Option<u8> {
        if uni_char.is_ascii() {
            Some(uni_char as u8)
        } else {
            ENCODING_TABLE_CP437.get(&uni_char).cloned()
        }
    }
}
