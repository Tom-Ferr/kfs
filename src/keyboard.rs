use crate::idt::{install_irq_routine, IntReg};
use crate::io::read_key;

pub static mut SHIFT_PRESSED: u8 = 0b0;
pub static mut PRESSED_KEY: u8 = 0x0;

const L_SHIFT: u8 = 0x2A;
const R_SHIFT: u8 = 0x36;
const L_SHIFT_RELEASE: u8 = 0x2A + 0x80;
const R_SHIFT_RELEASE: u8 = 0x36 + 0x80;
const CAPS_LOCK:u8 = 0x3A;

fn keyboard_handler(regs: *const IntReg){
    let scan_code = read_key();
    unsafe{
        if scan_code == L_SHIFT || scan_code == R_SHIFT{
            SHIFT_PRESSED |= 0b1;
        }
        else if scan_code == L_SHIFT_RELEASE || scan_code == R_SHIFT_RELEASE{
            SHIFT_PRESSED &= 0b10;
        }
        else if scan_code == CAPS_LOCK{
            SHIFT_PRESSED ^= 0b10;
        }
            
    }
}

pub fn init_keyboard(){
    install_irq_routine(1, keyboard_handler);
}