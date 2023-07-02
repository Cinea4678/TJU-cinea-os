use x86::io::outb;

#[repr(u16)]
enum IoPort {
    Com1 = 0x3F8
}

pub fn qemu_print(content: &str) {
    for ch in content.as_bytes() {
        unsafe { outb(IoPort::Com1 as u16, *ch); };
    }
}