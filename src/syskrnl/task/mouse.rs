use core::pin::Pin;
use core::task::{Context, Poll};

use conquer_once::spin::OnceCell;
use crossbeam::queue::ArrayQueue;
use futures_util::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use x86::io::inb;

use crate::syskrnl::gui::cursor::MOUSE_CURSOR;

static MOUSE_PACKAGE_QUEUE: OnceCell<ArrayQueue<MousePackage>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct MousePackage {
    middle_btn: bool,
    right_btn: bool,
    left_btn: bool,
    delta_x: i32,
    delta_y: i32,
}

impl MousePackage {
    pub fn new(byte_1: u8, byte_2: u8, byte_3: u8) -> Option<Self> {
        if byte_1 & 0xC0 > 0 || byte_1 & 0x8 == 0 {
            None
        } else {
            let mut delta_x = byte_2 as i32;
            if byte_1 & 0x10 > 0 {
                delta_x = delta_x as i8 as i32;
            }
            let mut delta_y = byte_3 as i32;
            if byte_1 & 0x20 > 0 {
                delta_y = delta_y as i8 as i32;
            }
            delta_y *= -1; // 坐标体系不一致，需要反转
            let middle_btn = byte_1 & 0x4 > 0;
            let right_btn = byte_1 & 0x2 > 0;
            let left_btn = byte_1 & 0x1 > 0;
            Some(MousePackage {
                middle_btn,
                right_btn,
                left_btn,
                delta_x,
                delta_y,
            })
        }
    }
    pub fn middle_btn(&self) -> bool {
        self.middle_btn
    }
    pub fn right_btn(&self) -> bool {
        self.right_btn
    }
    pub fn left_btn(&self) -> bool {
        self.left_btn
    }
    pub fn delta_x(&self) -> i32 {
        self.delta_x
    }
    pub fn delta_y(&self) -> i32 {
        self.delta_y
    }
}

/// 鼠标中断处理函数
pub(crate) fn mouse_interrupt_handler() {
    let b1 = unsafe { inb(0x60) };
    let b2 = unsafe { inb(0x60) };
    let b3 = unsafe { inb(0x60) };
    if let Some(package) = MousePackage::new(b1, b2, b3) {
        add_package(package);
    }
    // unsafe { pics::PICS.lock().notify_end_of_interrupt(0x12) };
}

pub(crate) fn add_package(package: MousePackage) {
    if let Ok(queue) = MOUSE_PACKAGE_QUEUE.try_get() {
        if let Err(_) = queue.push(package) {
            debugln!("警告：鼠标数据包队列已满; 正在丢弃数据包");
        } else {
            WAKER.wake();
        }
    } else {
        debugln!("警告：鼠标数据包队列尚未初始化");
    }
}

pub struct PackageStream {
    _private: (),
}

impl PackageStream {
    pub fn new() -> Self {
        MOUSE_PACKAGE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("PackageStream::new 只应当被调用一次哦");
        PackageStream { _private: () }
    }
}

impl Stream for PackageStream {
    type Item = MousePackage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = MOUSE_PACKAGE_QUEUE.try_get().expect("未初始化");

        if let Some(package) = queue.pop() {
            return Poll::Ready(Some(package));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(package) => {
                WAKER.take();
                Poll::Ready(Some(package))
            },
            None => Poll::Pending
        }
    }
}

pub async fn mouse_handler() {
    let mut packages = PackageStream::new();

    let mut btn_down = false;

    while let Some(package) = packages.next().await {
        // 处理package
        // debugln!("Mouse Event! x:{}, y:{}",package.delta_x,package.delta_y)
        MOUSE_CURSOR.lock().handle_change(package.delta_x, package.delta_y);
        if btn_down && !package.left_btn {
            btn_down = false;
            // handler
        } else if !btn_down && package.left_btn {
            btn_down = true;
        }

    }
}


