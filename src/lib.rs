#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
mod utils;
mod key_handlers;
mod commands;

use io::*;
use key_handlers::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static ASCII_ART: &[u8] = b"
           +++++[>++[>+>+        ++>++++>++++>++++>++++++
          >++++++>+++++++        ++>+++++++++<<<<<<<<<-]>>
         >+>+>+> >>>+[<]<        -]>>       >++>-->>+>>++>+
        >--<<<<  <<<.....         .>            ....<......
       ...>...   <<.>....                       >.>>>>>.<.
       <<<<..     ..<....                      >..>>>>>.<
      .<<<<.      >>>.<<.                     >>>>>.<.<
      <<<<<       <.>...>                    >>>.>>>.
     <<<.<        <<<..>>                  .>>>>>.<
    <.<<<         <<...>>                 >>>.<<<
   <..<.          ...>...               <<.>..>.
   >>.<.<<...>>...<<...>>...<         <....>>..
  .<<<.>.>>..>.<<.......<....        .....>...
                 <<.>...            .....>...
                 <......           .>>>.<<..
                 <<.>...          .....>...<......>.>>.<.<<<
                 .>......        ..>>...<<....>>.....>.<..>.

                 

                Please, press \'ENTER\' to check bonuses
";

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    
    clear_vga();
    
    enable_cursor(false);

    let offset = put_vga_string(ASCII_ART, 0);

    set_cursor(offset);

    loop {
        let scan_code = read_key();
        if scan_code == 0x1C {
            clear_vga();
            set_cursor(0);
            enable_cursor(true);
            break;
        }
    }

    let mut offset: u32 = 0;
    static mut SHIFT_PRESSED: u8 = 0b0;
    const L_SHIFT: u8 = 0x2A;
    const R_SHIFT: u8 = 0x36;
    const L_SHIFT_RELEASE: u8 = 0x2A + 0x80;
    const R_SHIFT_RELEASE: u8 = 0x36 + 0x80;
    const CAPS_LOCK:u8 = 0x3A;
    loop {
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
            match scan_code {
                0x0E => handle_backspace(&mut offset),
                0x4B => handle_left_arrow(&mut offset),
                0x4D => handle_right_arrow(&mut offset),
                0x53 => handle_delete(&mut offset),
                0x1C => handle_enter(&mut offset),
                _ => handle_character(scan_code, SHIFT_PRESSED, &mut offset),
            }
            
        }
    }
}