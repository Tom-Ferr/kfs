use crate::malloc::{kmalloc, kfree};
use crate::io::{scan_code_to_ascii, read_key};

#[macro_export]
macro_rules! get_reg {
    ($reg:ident) => {{
        let reg_value: u32;
        // unsafe {
            core::arch::asm!(concat!("mov {}, ", stringify!($reg)), out(reg) reg_value);
        // }
        reg_value as *const u32
    }};
}


pub fn buffer_count(mut offset: u32) -> u32{
    let mut count: u32 = 0;
    let vga_buffer = crate::io::VGA_BUFFER as *mut u8;

    unsafe{

        while *vga_buffer.offset(offset as isize) != b'\0'{
            count += 1;
            offset += 2;
        }
    }
    count 
}

pub fn strlen(string: *const u8) -> usize{
    let mut count: usize = 0;
    let mut offset = 0;

    unsafe{

        while *string.offset(offset as isize) != b'\0'{
            count += 1;
            offset += 1;
        }
    }
    count 
}

pub fn move_buffer_left(mut offset: u32){
    let vga_buffer = crate::io::VGA_BUFFER as *mut u8;

    
    let mut count = buffer_count(offset);

    unsafe{
        while count > 0{
                
            *vga_buffer.offset(offset as isize) = *vga_buffer.offset(offset as isize + 2);
            *vga_buffer.offset(offset as isize + 1) = *vga_buffer.offset(offset as isize + 3);
            count -= 1;
            offset += 2;
        }
    }
}

pub fn move_buffer_right(mut offset: u32){
    let vga_buffer = crate::io::VGA_BUFFER as *mut u8;

    
    let mut count = buffer_count(offset) + 1;
    
        offset += count * 2;
        unsafe{
            while count > 0{
    
                *vga_buffer.offset(offset as isize + 2) = *vga_buffer.offset(offset as isize);
                *vga_buffer.offset(offset as isize + 3) = *vga_buffer.offset(offset as isize + 1);
                count -= 1;
                offset -= 2;
            }
            *vga_buffer.offset(offset as isize + 2) = *vga_buffer.offset(offset as isize);
            *vga_buffer.offset(offset as isize + 3) = *vga_buffer.offset(offset as isize + 1);
        }
}

pub fn memcpy(source: *mut u8, dest: *mut u8, nbytes: u32) {
    unsafe{

        for i in 0..nbytes {
            *dest.offset(i as isize) = *source.offset(i as isize);
        }
    }
}

pub fn vga_strcmp(offset: u32, string: &[u8]) -> bool {
    let vga_buffer = crate::io::VGA_BUFFER as *const u8;
    let mut size = 0;
    unsafe {
        for (i, &byte) in string.iter().enumerate() {
            let char_byte = *vga_buffer.offset((offset as usize + (i * 2)) as isize);
            if char_byte != byte {
                return false;
            }
            size = i;
        }
        if *vga_buffer.offset((offset as usize + ((size + 1) * 2)) as isize) != 0x0{
            return false;
        }
    }
    true
}

pub fn get_line() -> Result<*const u8, ()> {
    let mut offset: u32 = 0;
    let mut size: u32 = 80;
    let mut buffer: Option<*mut u8> = None;
    loop{
        if let  Some(addr) = kmalloc(size as usize){
            if !buffer.is_none(){
                let prev = buffer.unwrap();
                memcpy(prev as *mut u8, addr as *mut u8, offset);
                kfree(prev as u32);
            }
            buffer = Some(addr as *mut u8);
            unsafe{

                for i in offset..size{
                    loop{
                        let scan_code = read_key();
                        if scan_code == 0x1C{
                            *(buffer.as_ref().unwrap()).offset(i as isize) = b'\0';
                            return Ok(buffer.unwrap() as *const u8);
                        }
                        if let Some(character) = scan_code_to_ascii(scan_code, crate::keyboard::SHIFT_PRESSED) {
                            *(buffer.as_ref().unwrap()).offset(i as isize) = character;
                            break;
                        }
                    }
                }
            }
            offset = size;
            size *= 2;
        }
        else {
            return Err(());
        }
    }
}