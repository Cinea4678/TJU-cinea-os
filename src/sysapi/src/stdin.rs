use alloc::string::String;
use crate::event::getch;

pub fn get(n: usize, buf: &mut [char]) -> usize {
    let size = n.min(buf.len());
    for i in 0..size{
        buf[i] = getch(true);
    }
    size
}

pub fn get_line(buf:&mut [char], with_new_line: bool) -> usize {
    let max_n = buf.len();
    for i in 0..max_n{
        let ch = getch(true);
        if ch!='\n'{
            buf[i]=ch;
        }else if with_new_line{
            buf[i]=ch;
            return i+1;
        }else{
            return i;
        }
    }
    max_n
}

pub fn get_line_string(with_new_line: bool) -> String {
    let mut str = String::new();
    loop {
        let ch = getch(true);
        if ch!='\n'{
            str.push(ch);
        }else if with_new_line{
            str.push(ch);
            break;
        }else{
            break;
        }
    }
    str
}