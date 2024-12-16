use core::arch::asm;

/// VGA Ports
const VGA_INDEX_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;

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
        *vga_buffer.offset(offset as isize + 1) = 0xa; // Light green text on black
}

pub fn put_vga_string(string: &[u8]) {
    let mut row = 0;
    let mut col = 0;

    for &byte in string {
        unsafe {
            if byte == b'\n' {
                row += 1;
                col = 0;
                continue;
            }
            let offset = (row * 80 + col) * 2; // VGA buffer is 80 columns wide
            put_vga_char(byte, offset);
            col += 1;
        }
    }
}

pub fn clear_vga() {

    unsafe{
        for i in 0..(25 * 80) {
            put_vga_char(b' ', i * 2);
        }
    }
}