use x86_64::instructions::interrupts;

pub fn dispatcher(event_id: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    interrupts::without_interrupts(|| {
        match event_id {}
    })
}