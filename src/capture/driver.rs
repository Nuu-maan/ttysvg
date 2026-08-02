use std::time::Duration;

use anyhow::Result;

use crate::capture::Session;
use crate::tape::{Key, Op};

pub fn run(session: &mut Session, ops: &[Op], type_delay: Duration) -> Result<()> {
    for op in ops {
        match op {
            Op::Type(text) => {
                for ch in text.chars() {
                    let mut buf = [0u8; 4];
                    session.send(ch.encode_utf8(&mut buf).as_bytes())?;
                    if !type_delay.is_zero() {
                        std::thread::sleep(type_delay);
                    }
                }
            }
            Op::Key(key) => session.send(key.bytes())?,
            Op::Sleep(d) => std::thread::sleep(*d),
            Op::Wait { text, timeout } => session.wait_for(text, *timeout)?,
        }
    }
    Ok(())
}

const CTRL: [[u8; 1]; 26] = [
    [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [11], [12], [13], [14], [15], [16], [17],
    [18], [19], [20], [21], [22], [23], [24], [25], [26],
];

impl Key {
    pub fn bytes(&self) -> &'static [u8] {
        match self {
            Key::Enter => b"\r",
            Key::Tab => b"\t",
            Key::Backspace => b"\x7f",
            Key::Escape => b"\x1b",
            Key::Space => b" ",
            Key::Up => b"\x1b[A",
            Key::Down => b"\x1b[B",
            Key::Right => b"\x1b[C",
            Key::Left => b"\x1b[D",
            Key::Ctrl(c) => {
                let lower = c.to_ascii_lowercase();
                if lower.is_ascii_lowercase() {
                    &CTRL[(lower as u8 - b'a') as usize]
                } else {
                    b""
                }
            }
        }
    }
}
