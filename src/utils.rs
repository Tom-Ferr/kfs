
pub fn buffer_count(mut offset: u32) -> u32{
    let mut count: u32 = 0;
    let vga_buffer = 0xb8000 as *mut u8;

    unsafe{

        while *vga_buffer.offset(offset as isize) != b'\0'{
            count += 1;
            offset += 2;
        }
    }
    count 
}

pub fn move_buffer_left(mut offset: u32){
    let vga_buffer = 0xb8000 as *mut u8;

    
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
    let vga_buffer = 0xb8000 as *mut u8;

    
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