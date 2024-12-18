use crate::io::*;

pub const COMMANDS: [(&[u8], fn(u32)); 10] = [
    (b"dark", |_offset: u32| color_mode(false)),
    (b"light", |_offset: u32| color_mode(true)),
    (b"cyan", |offset: u32| paint(offset, Color::Cyan as u8)),
    (b"yellow", |offset: u32| paint(offset, Color::Yellow as u8)),
    (b"magenta", |offset: u32| paint(offset, Color::Magenta as u8)),
    (b"red", |offset: u32| paint(offset, Color::Red as u8)),
    (b"green", |offset: u32| paint(offset, Color::Green as u8)),
    (b"blue", |offset: u32| paint(offset, Color::Blue as u8)),
    (b"pink", |offset: u32| paint(offset, Color::Pink as u8)),
    (b"white", |offset: u32| paint(offset, Color::White as u8)),
];