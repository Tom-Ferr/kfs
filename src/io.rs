use core::arch::asm;

use crate::utils;

/// VGA Ports
const VGA_INDEX_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;
const VGA_OFFSET_LOW: u16 = 0x0f;
const VGA_OFFSET_HIGH: u16 = 0x0e;

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

static mut TEXT_COLOR: u8 = Color::LightGreen as u8;
static mut BACKGROUND_COLOR: u8 = Color::Black as u8;

pub fn paint(mut offset: u32, color: u8) {
    unsafe{
        let vga_buffer = 0xb8000 as *mut u8;
        let count = utils::buffer_count(offset);
        for _ in 0..count{
            *vga_buffer.offset(offset as isize + 1) = BACKGROUND_COLOR << 4 | color;
            offset += 2;
        }
    }
}

pub fn color_mode(light: bool) {
    unsafe{
        let prev_text_color: u8 = TEXT_COLOR;
        if light == false{
            TEXT_COLOR = Color::LightGreen as u8;
            BACKGROUND_COLOR = Color::Black as u8;
        }
        else{
            TEXT_COLOR = Color::LightCyan as u8;
            BACKGROUND_COLOR = Color::DarkGray as u8;
        }
        let vga_buffer = 0xb8000 as *mut u8;
        let mut offset: u32 = 0;
        for _ in 0..(25 * 80) {
            let current: u8 = *vga_buffer.offset(offset as isize + 1) & 0b1111;
            let mut color = TEXT_COLOR;
            if current != prev_text_color {
                color = current;
            }
            *vga_buffer.offset(offset as isize + 1) = BACKGROUND_COLOR << 4 | color;
            offset += 2;
        }

    }
}

/// Write a byte to an I/O port
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value);
}

/// Read a byte from an I/O port
unsafe fn inb(port: u16) -> u8 {
    let mut value: u8;
    asm!("in al, dx", out("al") value, in("dx") port);
    value
}

pub fn enable_cursor(swicth: bool) {
    unsafe{
        // Select Cursor Start Register (0x0A)
        outb(VGA_INDEX_PORT, 0x0A);
        
        // Read the current value of the Cursor Start Register
        let cursor_start = inb(VGA_DATA_PORT);
        
        if swicth == true{
            // Unset bit 5 (most significant bit) to enable the cursor
            outb(VGA_DATA_PORT, cursor_start ^ 0x20);
        }
        else{
            // Set bit 5 (most significant bit) to disable the cursor
            outb(VGA_DATA_PORT, cursor_start | 0x20);
        }
    }
}

pub fn read_key() -> u8 {
    unsafe {
        // Wait until bit 0 of status register (0x64) is set
        while inb(0x64) & 0x01 == 0 {}
        
        // Read the scan code from data port (0x60)
        inb(0x60)
    }
}

pub unsafe fn put_vga_char(byte: u8, offset: u32) {
        let vga_buffer = 0xb8000 as *mut u8;
        *vga_buffer.offset(offset as isize) = byte;
        *vga_buffer.offset(offset as isize + 1) = BACKGROUND_COLOR << 4 | TEXT_COLOR;
}

pub fn put_vga_string(string: &[u8], mut offset: u32) -> u32 {
    let mut row = 0;
    let mut col = 0;

    for &byte in string {
        unsafe {
            if byte == b'\n' {
                row += 1;
                col = 0;
                continue;
            }
            put_vga_char(byte, offset);
            col += 1;
            offset = (row * 80 + col) * 2; // VGA buffer is 80 columns wide
        }
    }
    offset
}

pub fn clear_vga() {

    unsafe{
        for i in 0..(25 * 80) {
            put_vga_char(b'\0', i * 2);
        }
        set_cursor(0);
    }
}

pub fn set_cursor(mut offset: u32)
{
    offset /= 2;
    unsafe{
        outb(VGA_INDEX_PORT, VGA_OFFSET_HIGH as u8);
        outb(VGA_DATA_PORT, (offset >> 8) as u8);
        outb(VGA_INDEX_PORT, VGA_OFFSET_LOW as u8);
        outb(VGA_DATA_PORT, (offset & 0xff) as u8);
    }
}

pub fn get_cursor() -> u32 {
    let mut offset: u32 = 0;
    unsafe{

        outb(VGA_INDEX_PORT, VGA_OFFSET_HIGH as u8);
        offset += (inb(VGA_DATA_PORT) as u32) << 8;
        outb(VGA_INDEX_PORT, VGA_OFFSET_LOW as u8);
        offset += inb(VGA_DATA_PORT) as u32;
    }
    offset * 2
}

pub fn scan_code_to_ascii(scan_code: u8, shift_key: u8) -> Option<u8> {

    match scan_code {
        0x1E if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'a'),
        0x30 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'b'),
        0x2E if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'c'),
        0x20 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'd'),
        0x12 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'e'),
        0x21 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'f'),
        0x22 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'g'),
        0x23 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'h'),
        0x17 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'i'),
        0x24 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'j'),
        0x25 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'k'),
        0x26 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'l'),
        0x32 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'm'),
        0x31 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'n'),
        0x18 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'o'),
        0x19 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'p'),
        0x10 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'q'),
        0x13 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'r'),
        0x1F if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b's'),
        0x14 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b't'),
        0x16 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'u'),
        0x2F if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'v'),
        0x11 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'w'),
        0x2D if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'x'),
        0x15 if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'y'),
        0x2C if shift_key & 3 == 0 || shift_key & 3 == 3 => Some(b'z'),
        
        0x1E if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'A'),
        0x30 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'B'),
        0x2E if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'C'),
        0x20 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'D'),
        0x12 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'E'),
        0x21 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'F'),
        0x22 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'G'),
        0x23 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'H'),
        0x17 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'I'),
        0x24 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'J'),
        0x25 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'K'),
        0x26 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'L'),
        0x32 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'M'),
        0x31 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'N'),
        0x18 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'O'),
        0x19 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'P'),
        0x10 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'Q'),
        0x13 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'R'),
        0x1F if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'S'),
        0x14 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'T'),
        0x16 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'U'),
        0x2F if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'V'),
        0x11 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'W'),
        0x2D if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'X'),
        0x15 if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'Y'),
        0x2C if shift_key & 3 == 2 || shift_key & 3 == 1 => Some(b'Z'),
        
        0x02 if shift_key & 1 == 0 => Some(b'1'),
        0x03 if shift_key & 1 == 0 => Some(b'2'),
        0x04 if shift_key & 1 == 0 => Some(b'3'),
        0x05 if shift_key & 1 == 0 => Some(b'4'),
        0x06 if shift_key & 1 == 0 => Some(b'5'),
        0x07 if shift_key & 1 == 0 => Some(b'6'),
        0x08 if shift_key & 1 == 0 => Some(b'7'),
        0x09 if shift_key & 1 == 0 => Some(b'8'),
        0x0A if shift_key & 1 == 0 => Some(b'9'),
        0x0B if shift_key & 1 == 0 => Some(b'0'),

        0x02 if shift_key & 1 == 1 => Some(b'!'),
        0x03 if shift_key & 1 == 1 => Some(b'@'),
        0x04 if shift_key & 1 == 1 => Some(b'#'),
        0x05 if shift_key & 1 == 1 => Some(b'$'),
        0x06 if shift_key & 1 == 1 => Some(b'%'),
        0x07 if shift_key & 1 == 1 => Some(b'^'),
        0x08 if shift_key & 1 == 1 => Some(b'&'),
        0x09 if shift_key & 1 == 1 => Some(b'*'),
        0x0A if shift_key & 1 == 1 => Some(b'('),
        0x0B if shift_key & 1 == 1 => Some(b')'),
        
        0x39 => Some(b' '),
        
        // Ignore key releases
        0x80..=0xFF => None,
        
        // Unmapped keys
        _ => None,
    }
}

pub fn put_keyboard_input(character: u8, mut offset: u32) -> u32{
    if offset >= 25 * 80 * 2 {
        offset = scroll_ln(offset);
    }
    unsafe{put_vga_char(character, offset);}
    set_cursor(offset + 2);
    offset + 2
}

pub fn get_row_from_offset(offset: u32) -> u32 {
    offset / (2 * 80)
}

pub fn get_offset(col: u32, row: u32) -> u32 {
    2 * (row * 80 + col)
}

pub fn move_offset_to_new_line(offset: u32) -> u32 {
    get_offset(0, get_row_from_offset(offset) + 1)
}

pub fn scroll_ln(offset: u32) -> u32 {
    let vga_buffer = 0xb8000 as *mut u8;
    unsafe{
        utils::memcpy(vga_buffer.offset(get_offset(0, 1) as isize), vga_buffer.offset(get_offset(0, 0) as isize), 80 * (25 - 1) * 2);
        
        for col in 0..80 {
            put_vga_char(b' ', get_offset(col, 25 - 1));
        }
    }
    offset - 2 * 80
}