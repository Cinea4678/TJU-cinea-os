#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use lazy_static::lazy_static;
use spin::Mutex;
use tinyrand::{Rand, Seeded, Wyrand};
use ufmt::uwrite;

use cinea_os_sysapi::{allocator, entry_point, gui, rgb888};
use cinea_os_sysapi::event::{GUI_EVENT_EXIT, GUI_EVENT_UNDK_KEY, register_gui_keyboard, wait_gui_event};
use cinea_os_sysapi::fs::{open, read, read_all_from_path};
use cinea_os_sysapi::gui::{load_font, remove_window_gui, WindowWriter};
use cinea_os_userspace::print;
use cinea_os_userspace::std::StringWriter;

entry_point!(main);

#[global_allocator]
static ALLOCATOR: allocator::UserProcAllocator = allocator::UserProcAllocator;

type Map = [[i32; 4]; 4];

static SEED: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref RAND: Mutex<Wyrand> = Mutex::new(Wyrand::seed(SEED.load(Ordering::SeqCst)));
}

fn init_map(map: &mut Map) {
    let seed = open("/dev/uptime", false).unwrap();
    let mut buf = [0u8; 8];
    read(seed, &mut buf).unwrap();
    let seed = f64::from_le_bytes(buf);
    let seed = (seed * 10000.0) as u64;

    SEED.store(seed, Ordering::SeqCst);
    let mut rand = RAND.lock();

    for _ in 0..3 {
        let mut x;
        let mut y;
        loop {
            x = rand.next_usize() % 4;
            y = rand.next_usize() % 4;
            if map[x][y] >= 0 { continue; }
            break;
        }
        map[x][y] = 0;
    }
}

fn count_last_blocks(map: &Map) -> usize {
    let mut res = 0;
    for i in map {
        for j in i {
            if *j < 0 { res += 1 }
        }
    }
    res
}

fn check_fail(map: &Map) -> bool {
    count_last_blocks(map) < 2
}

fn random_next(map: &mut Map) {
    if count_last_blocks(map) < 2 { return; }
    let mut rand = RAND.lock();
    for _ in 0..2 {
        let mut x;
        let mut y;
        loop {
            x = rand.next_usize() % 4;
            y = rand.next_usize() % 4;
            if map[x][y] >= 0 { continue; }
            break;
        }
        map[x][y] = 0;
    }
}

/// 合并一行。
/// 注意：这里的一行是从右往左合并的，也就是说最后肯定是往小序号的方向合并
fn merge_line(line: &mut [i32; 4]) -> usize {
    let mut score = 0;

    // print!("B {},{},{},{}\n", line[0], line[1], line[2], line[3]);

    // 先全部移到一侧
    for i in 1..4 {
        let mut j = i;
        while j > 0 && line[j] >= 0 && line[j - 1] < 0 {
            line[j - 1] = line[j];
            line[j] = -1;
            j -= 1;
        }
    }

    // 接下来，尝试进行合并
    for i in 0..3 {
        while line[i]>=0 && line[i] == line[i + 1] {
            line[i] += 1;
            score += 1 << (line[i] + 2);
            for j in (i + 1)..3 {
                line[j] = line[j + 1];
            }
            line[3] = -1;
        }
    }

    // print!("A {},{},{},{}\n", line[0], line[1], line[2], line[3]);

    score
}

fn slide(map: &mut Map, direction: usize) -> usize {
    let mut score = 0;

    let mut buf = [-1i32; 4];

    match direction {
        0 => {
            // 上
            for j in 0..4 {
                for i in 0..4 {
                    buf[i] = map[i][j];
                }
                score += merge_line(&mut buf);
                for i in 0..4 {
                    map[i][j] = buf[i];
                }
            }
        }
        1 => {
            // 下
            for j in 0..4 {
                for i in 0..4 {
                    buf[3 - i] = map[i][j];
                }
                score += merge_line(&mut buf);
                for i in 0..4 {
                    map[i][j] = buf[3 - i];
                }
            }
        }
        2 => {
            // 左
            for i in 0..4 {
                buf.copy_from_slice(&map[i]);
                score += merge_line(&mut buf);
                map[i].copy_from_slice(&buf);
            }
        }
        3 => {
            // 右
            for i in 0..4 {
                buf.copy_from_slice(&map[i]);
                buf.reverse();
                score += merge_line(&mut buf);
                buf.reverse();
                map[i].copy_from_slice(&buf);
            }
        }
        _ => unreachable!()
    }


    score
}

fn main(_args: &[&str]) {
    print!("2048进程已启动\n");
    if !load_font("Vonwaon", "/sys/ast/VonwaonBitmap-16px.ttf") {
        panic!("Load font failed")
    }

    let mut window_instance: WindowWriter = gui::init_window_gui("2048", rgb888!(0xfaf8efu32)).expect("获取窗口实例失败");

    let board_raw = read_all_from_path("/sys/2048/background.bmp").unwrap();
    let board = WindowWriter::resolve_img(&board_raw).unwrap();

    let chess_file_name = [
        "/sys/2048/2048_4.bmp",
        "/sys/2048/2048_8.bmp",
        "/sys/2048/2048_16.bmp",
        "/sys/2048/2048_32.bmp",
        "/sys/2048/2048_64.bmp",
        "/sys/2048/2048_128.bmp",
        "/sys/2048/2048_256.bmp",
        "/sys/2048/2048_512.bmp",
        "/sys/2048/2048_1024.bmp",
        "/sys/2048/2048_2048.bmp",
    ];
    let _ = read_all_from_path("/sys/2048/2048_2048.bmp").unwrap();
    let chess: Vec<_> = chess_file_name
        .iter()
        .map(|file_name| read_all_from_path(file_name).unwrap())
        .map(|raw| WindowWriter::resolve_img(&raw).unwrap())
        .collect();

    let mut map: Map = [[-1; 4]; 4];
    let mut score = 0;

    let mut render = |map: &Map, score: &usize| {
        window_instance.display_resolved(0, 0, &board);
        for i in 0..4 {
            for j in 0..4 {
                if map[i][j] >= 0 {
                    window_instance.display_resolved(9 + i as i32 * 40, 9 + j as i32 * 40, &chess[map[i][j] as usize])
                }
            }
        }
        let mut score_text = StringWriter::new();
        uwrite!(score_text, "{}", score).unwrap();
        window_instance.display_font_string("当前得分", "Vonwaon", 30, 180, 16.0, 16, rgb888!(0));
        window_instance.display_font_string(score_text.value().as_str(), "Vonwaon", 48, 180, 16.0, 16, rgb888!(0));
    };

    init_map(&mut map);

    register_gui_keyboard();

    loop {
        render(&map, &score);
        let (code, _arg0, _arg1, _arg2) = wait_gui_event();
        match code {
            GUI_EVENT_EXIT => {
                remove_window_gui(window_instance);
                break;
            }
            GUI_EVENT_UNDK_KEY => {
                let key = _arg0;
                match key {
                    5 => {
                        score += slide(&mut map, 0);
                        random_next(&mut map);
                    }
                    2 => {
                        score += slide(&mut map, 1);
                        random_next(&mut map);
                    }
                    3 => {
                        score += slide(&mut map, 2);
                        random_next(&mut map);
                    }
                    4 => {
                        score += slide(&mut map, 3);
                        random_next(&mut map);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        if check_fail(&map) {
            print!("\n[来自2048] 游戏结束，您的得分是：{}", score);
            remove_window_gui(window_instance);
            break;
        }
    }
}
