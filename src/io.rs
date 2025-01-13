use core::arch::asm;
use core::fmt;

use crate::utils;
use crate::get_reg;

/// VGA Ports
const VGA_INDEX_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;
const VGA_OFFSET_LOW: u16 = 0x0f;
const VGA_OFFSET_HIGH: u16 = 0x0e;

pub const VGA_BUFFER: u32 = 0xC00B8000;

#[macro_export]
#[allow(dead_code)]
macro_rules! printf {
    ($($arg:tt)*) => {
        $crate::io::_print_fmt_str(core::format_args!($($arg)*));
    };
}
#[macro_export]
#[allow(dead_code)]
macro_rules! printk {
    ($level:ident, $($arg:tt)*) => {
        $crate::io::_print_fmt_log($crate::io::LogLevel::$level, core::format_args!($($arg)*));
    };
    
    ($($arg:tt)*) => {
        $crate::io::_print_fmt_log($crate::io::LogLevel::INFO, core::format_args!($($arg)*));
    };
}

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
#[allow(dead_code)]
const INFO_TEXT_COLOR: Color = Color::White;

#[allow(dead_code)]
const INFO_BG_COLOR: Color = Color::Black;

#[allow(dead_code)]
const WARNING_TEXT_COLOR: Color = Color::Black;

#[allow(dead_code)]
const WARNING_BG_COLOR: Color = Color::Yellow;

#[allow(dead_code)]
const ERROR_TEXT_COLOR: Color = Color::White;

#[allow(dead_code)]
const ERROR_BG_COLOR: Color = Color::Red;

#[allow(dead_code)]
const DEBUG_TEXT_COLOR: Color = Color::Green;

#[allow(dead_code)]
const DEBUG_BG_COLOR: Color = Color::Black;

#[allow(dead_code)]
pub enum LogLevel {
    INFO,
    WARNING,
    ERROR,
    DEBUG,
}

static mut TEXT_COLOR: u8 = Color::LightGreen as u8;
static mut BACKGROUND_COLOR: u8 = Color::Black as u8;

pub fn paint(mut offset: u32, color: u8) {
    unsafe{
        let vga_buffer = VGA_BUFFER as *mut u8;
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
        let vga_buffer = VGA_BUFFER as *mut u8;
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
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value);
}

/// Write a word to an I/O port
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value);
}

/// Read a byte from an I/O port
pub unsafe fn inb(port: u16) -> u8 {
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

pub struct Writer;

impl Writer {
    pub fn new() -> Self {
        Writer
    }
}

impl Writer {

    pub fn write_string(string: &[u8]){
        put_vga_string(string);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Writer::write_string(s.as_bytes());
        Ok(())
    }
}

pub fn _print_fmt_str(args: core::fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = Writer::new();
    let _ = writer.write_fmt(args);
}

pub fn _print_fmt_log(level: LogLevel, args: core::fmt::Arguments) {
    let (text_color, background_color, level_prefix) = match level {
        LogLevel::INFO => (INFO_TEXT_COLOR, INFO_BG_COLOR, "\n[INFO] "),
        LogLevel::WARNING => (WARNING_TEXT_COLOR, WARNING_BG_COLOR, "\n[WARNING] "),
        LogLevel::ERROR => (ERROR_TEXT_COLOR, ERROR_BG_COLOR, "\n[ERROR] "),
        LogLevel::DEBUG => (DEBUG_TEXT_COLOR, DEBUG_BG_COLOR, "\n[DEBUG] "),
    };

    unsafe{
        let prev_text_color = TEXT_COLOR;
        let prev_background_color = BACKGROUND_COLOR;

        TEXT_COLOR = text_color as u8;
        BACKGROUND_COLOR = background_color as u8;
        
        put_vga_string(level_prefix.as_bytes());
        
        _print_fmt_str(args);
        
        TEXT_COLOR = prev_text_color;
        BACKGROUND_COLOR = prev_background_color;
    }
}

pub unsafe fn put_vga_char(byte: u8, offset: u32) {
        let vga_buffer = VGA_BUFFER as *mut u8;
        *vga_buffer.offset(offset as isize) = byte;
        *vga_buffer.offset(offset as isize + 1) = BACKGROUND_COLOR << 4 | TEXT_COLOR;
}

pub fn put_vga_string(string: &[u8]) {
    
    let mut offset = get_cursor();
    for &byte in string {
        unsafe {
            if byte == b'\n' {
                offset = move_offset_to_new_line(offset);
                continue;
            }
            put_vga_char(byte, offset);
            offset += 2;
        }
    }
    set_cursor(offset);
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

        0x0C if shift_key & 1 == 0 => Some(b'-'),
        0x0D if shift_key & 1 == 0 => Some(b'='),
        0x1A if shift_key & 1 == 0 => Some(b'['),
        0x1B if shift_key & 1 == 0 => Some(b']'),
        0x2B if shift_key & 1 == 0 => Some(b'\\'),
        0x27 if shift_key & 1 == 0 => Some(b';'),
        0x28 if shift_key & 1 == 0 => Some(b'\''),
        0x29 if shift_key & 1 == 0 => Some(b'`'),
        0x33 if shift_key & 1 == 0 => Some(b','),
        0x34 if shift_key & 1 == 0 => Some(b'.'),
        0x35 if shift_key & 1 == 0 => Some(b'/'),

        0x0C if shift_key & 1 == 1 => Some(b'_'),
        0x0D if shift_key & 1 == 1 => Some(b'+'),
        0x1A if shift_key & 1 == 1 => Some(b'{'),
        0x1B if shift_key & 1 == 1 => Some(b'}'),
        0x2B if shift_key & 1 == 1 => Some(b'|'),
        0x27 if shift_key & 1 == 1 => Some(b':'),
        0x28 if shift_key & 1 == 1 => Some(b'\"'),
        0x29 if shift_key & 1 == 1 => Some(b'~'),
        0x33 if shift_key & 1 == 1 => Some(b'<'),
        0x34 if shift_key & 1 == 1 => Some(b'>'),
        0x35 if shift_key & 1 == 1 => Some(b'?'),

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
    let vga_buffer = VGA_BUFFER as *mut u8;
    unsafe{
        utils::memcpy(vga_buffer.offset(get_offset(0, 1) as isize), vga_buffer.offset(get_offset(0, 0) as isize), 80 * (25 - 1) * 2);
        
        for col in 0..80 {
            put_vga_char(b' ', get_offset(col, 25 - 1));
        }
    }
    offset - 2 * 80
}

pub fn stack_dump() {
    
    unsafe {
        let stack_base = get_reg!(ebp);
        let stack_pointer = get_reg!(esp);

        memory_dump(stack_pointer, stack_base);
    }

}

pub fn memory_dump(begin_pointer: *const u32, end_pointer: *const u32) {
    clear_vga();
    set_cursor(0);
    enable_cursor(false);
    
    unsafe {

        let mut iter = begin_pointer as *const u8;

        let end = end_pointer as *const u8;

        let mut offset = 0;

        const PUT: fn(u8, u32, u32) = |value: u8, offset: u32, entry: u32| {
            unsafe{

                if value > 32 as u8 && value < 127 {
                    put_vga_char(value, offset + (((15 * 3) + 4) * 2) + entry);
                    
                }
                else {
                    put_vga_char('.' as u8, offset + (((15 * 3) + 4) * 2) + entry);
                }
            }
        };

        while iter < end {
            for entry in (0..32).step_by(2) {
                if iter >= end {break}
                if entry == 0 {
                    printf!("{:?}:  {:02x}", iter, *iter);
                    offset = get_cursor();
                    PUT(*iter, offset, entry);
                }
                else {
                    printf!(" {:02x}", *iter);
                    PUT(*iter, offset, entry);
                }
                iter = iter.wrapping_add(1);
            }
            printf!("\n");
            if get_cursor() == 24 * 160 {
                printf!("Press \'ENTER\' to continue, \'ESC\' to quit");
                loop{
                    let scan_code = read_key();
                    if scan_code == 0x01{
                        clear_vga();
                        set_cursor(0);
                        enable_cursor(true);
                        return ;
                    }
                    else if scan_code == 0x1C{
                        clear_vga();
                        set_cursor(0);
                        break ;
                    }
                }
            }
        }
        printf!("Press \'ENTER\' to quit");
        loop{
            let scan_code = read_key();
            if scan_code == 0x1C{
                clear_vga();
                set_cursor(0);
                enable_cursor(true);
                break ;
            }
        }
    }

}