
use crate::utils::*;
use crate::io::*;

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
    if vga_strcmp(row_offset, b"yellow"){
        paint(row_offset, Color::Yellow as u8);
    }
    else if vga_strcmp(row_offset, b"magenta"){
        paint(row_offset, Color::Magenta as u8);
    }
    else if vga_strcmp(row_offset, b"cyan"){
        paint(row_offset, Color::Cyan as u8);
    }
    else if vga_strcmp(row_offset, b"red"){
        paint(row_offset, Color::Red as u8);
    }
    else if vga_strcmp(row_offset, b"white"){
        paint(row_offset, Color::White as u8);
    }
    else if vga_strcmp(row_offset, b"blue"){
        paint(row_offset, Color::Blue as u8);
    }
    else if vga_strcmp(row_offset, b"green"){
        paint(row_offset, Color::Green as u8);
    }
    else if vga_strcmp(row_offset, b"pink"){
        paint(row_offset, Color::Pink as u8);
    }
    else if vga_strcmp(row_offset, b"dark"){
        color_mode(false);
    }
    else if vga_strcmp(row_offset, b"light"){
        color_mode(true);
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