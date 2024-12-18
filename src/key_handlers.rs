
use crate::utils::*;
use crate::io::*;
use crate::commands::*;

pub fn handle_backspace(offset: &mut u32) {
    if *offset % 160 > 0 {
        *offset -= 2;
        move_buffer_left(*offset);
        set_cursor(*offset);
    }
}

pub fn handle_left_arrow(offset: &mut u32) {
    if *offset % 160 > 0 {
        *offset -= 2;
        set_cursor(*offset);
    }
}

pub fn handle_right_arrow(offset: &mut u32) {
    let row_offset = get_row_from_offset(*offset) * 160;
    if *offset % 160 < buffer_count(row_offset) * 2 {
        *offset += 2;
        set_cursor(*offset);
    }
}

pub fn handle_delete(offset: &mut u32) {
    let row_offset = get_row_from_offset(*offset) * 160;
    if *offset % 160 < buffer_count(row_offset) * 2 {
        move_buffer_left(*offset);
    }
}

pub fn handle_enter(offset: &mut u32) {
    let row_offset = get_row_from_offset(*offset) * 160;
    for (command, action) in COMMANDS.iter(){
        if vga_strcmp(row_offset, command){
            action(row_offset);
        }
    }
    *offset = move_offset_to_new_line(*offset);
    if *offset >= 25 * 80 * 2 {
        *offset = scroll_ln(*offset);
    }
    set_cursor(*offset);
}

pub fn handle_character(scan_code: u8, shift_pressed: u8, offset: &mut u32) {
    if let Some(character) = scan_code_to_ascii(scan_code, shift_pressed) {
        move_buffer_right(*offset);
        *offset = put_keyboard_input(character, *offset);
    }
}