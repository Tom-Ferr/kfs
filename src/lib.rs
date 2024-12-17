#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;

use io::*;

mod utils;

mod key_handlers;

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
    loop {
        let scan_code = read_key();
        unsafe{
            if scan_code == 0x2A || scan_code == 0x36{
                SHIFT_PRESSED |= 0b1;
            }
            else if scan_code == 0x2A + 0x80 || scan_code == 0x36 + 0x80{ //SHIFT RELEASE
                SHIFT_PRESSED &= 0b10;
            }
            else if scan_code == 0x3A{ //CAPS-LOCK
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