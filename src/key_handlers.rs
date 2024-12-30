
use crate::utils::*;
use crate::io::*;
use crate::commands::*;
use crate::screens::*;

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
    *offset = move_offset_to_new_line(*offset);
    set_cursor(*offset);
    for (command, action) in COMMANDS.iter(){
        if vga_strcmp(row_offset, command){
            action(row_offset);
        }
    }
    *offset = get_cursor();
    if *offset >= 25 * 80 * 2 {
        *offset = scroll_ln(*offset);
        set_cursor(*offset);
    }
}

pub fn handle_character(scan_code: u8, shift_pressed: u8, offset: &mut u32) {
    if let Some(character) = scan_code_to_ascii(scan_code, shift_pressed) {
        move_buffer_right(*offset);
        *offset = put_keyboard_input(character, *offset);
    }
}
pub fn handle_shortcuts(current_screen: & Screen) -> Option<Screen> {
    loop {
        let scan_code = read_key();
        match scan_code {
            0x2 if *current_screen != Screen::Screen1 => { return Some(Screen::Screen1);},
            0x3 if *current_screen != Screen::Screen2 => { return Some(Screen::Screen2);},
            0x4 if *current_screen != Screen::Screen3 => { return Some(Screen::Screen3);},
            0x26 if *current_screen == Screen::Screen2 => {clear_vga();}
            0x9D => break,
                _ => {},
            }
    }
    None
}