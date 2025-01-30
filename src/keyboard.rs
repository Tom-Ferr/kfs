use crate::idt::{install_irq_routine, IntReg};
use crate::io::read_key;

pub static mut SHIFT_PRESSED: u8 = 0b0;
pub static mut PRESSED_KEY: u8 = 0x0;
pub static mut CURRENT_LAYOUT: [u8; 94] = QWERTY_LAYOUT;

const L_SHIFT: u8 = 0x2A;
const R_SHIFT: u8 = 0x36;
const L_SHIFT_RELEASE: u8 = 0x2A + 0x80;
const R_SHIFT_RELEASE: u8 = 0x36 + 0x80;
const CAPS_LOCK:u8 = 0x3A;

const QWERTY_LAYOUT: [u8; 94] = [
    b'~', b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+',
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', b'|',
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'\"',
    b'Z', b'X', b'C', b'V', b'B', b'N', b'M', b'<', b'>', b'?',
    
    b'`', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=',
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\\',
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'',
    b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/',
];

const AZERTY_LAYOUT: [u8; 94] = [
    0xFD, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', 0xF8, b'+',  // Row 1: ² 1 2 3 4 5 6 7 8 9 0 ° +
    b'A', b'Z', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'^', b'$', 0x9C,  // Row 2: A Z E R T Y U I O P ^ $ £
    b'Q', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', b'M', b'%',              // Row 3: Q S D F G H J K L M %
    b'W', b'X', b'C', b'V', b'B', b'N', b'?', b'.', b'/', 0x15,                    // Row 4: W X C V B N ? . / §

    0xFD, b'&', 0x82, b'"', b'\'', b'(', b'-', 0x8A, b'_', 0x87, 0x85, b')', b'=',  // Row 1: & é " ' ( - è _ ç à ) = `
    b'a', b'z', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'^', b'$', b'*',   // Row 2: a z e r t y u i o p ^ $ *
    b'q', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b'm', 0x97,               // Row 3: q s d f g h j k l m ù
    b'w', b'x', b'c', b'v', b'b', b'n', b',', b';', b':', b'!'                      // Row 4: w x c v b n , ; : !
];

pub fn set_azerty(){
    unsafe{CURRENT_LAYOUT = AZERTY_LAYOUT};
}

pub fn set_querty(){
    unsafe{CURRENT_LAYOUT = QWERTY_LAYOUT};
}

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