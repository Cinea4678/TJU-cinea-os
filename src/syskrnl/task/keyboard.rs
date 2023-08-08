use core::pin::Pin;
use core::task::{Context, Poll};

use conquer_once::spin::OnceCell;
use crossbeam::queue::ArrayQueue;
use futures_util::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, layouts, ScancodeSet1};
use x86::io::inb;
use x86_64::instructions::interrupts;
use cinea_os_sysapi::event::*;

use crate::{debug, print, syskrnl};
use crate::syskrnl::event;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// 键盘中断处理函数
fn keyboard_interrupt_handler() {
    let scancode: u8 = unsafe { inb(0x60) };
    add_scancode(scancode);
}

pub fn init() {
    interrupts::without_interrupts(|| {
        syskrnl::interrupts::set_irq_handler(1, keyboard_interrupt_handler);
    });
}

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            debugln!("警告：键盘扫描码队列已满; 正在丢弃键盘输入");
        } else {
            WAKER.wake();
        }
    } else {
        debugln!("警告：键盘扫描码队列尚未初始化");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new 只应当被调用一次哦");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("未初始化");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop(){
            Some(scancode)=>{
                WAKER.take();
                Poll::Ready(Some(scancode))
            },
            None => Poll::Pending
        }
    }
}

pub async fn print_keypresses(){
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::MapLettersToUnicode);

    while let Some(scancode) = scancodes.next().await{
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode){
            if let Some(key) = keyboard.process_keyevent(key_event) {
                debugln!("{:?}",key);
                match key {
                    DecodedKey::Unicode(character) => {key_event_handler(character)}
                    DecodedKey::RawKey(key) => {debug!("undk:{:?}",key)}
                }
            }
        }
    }
}

fn key_event_handler(ch:char){
    event::EVENT_QUEUE.lock().wakeup_with_ret(KEYBOARD_INPUT, ch as usize);
}
