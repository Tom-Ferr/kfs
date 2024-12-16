#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;

use io::*;

mod utils;

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

                 

                Please, press \'ENTER\' to check the bonuses
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
    loop {
        let scan_code = read_key();
        static mut SHIFT_PRESSED: bool = false;
        unsafe{

            if scan_code == 0x2A || scan_code == 0x36{
                SHIFT_PRESSED = true;
            }
            else if scan_code == 0x2A + 0x80 || scan_code == 0x36 + 0x80{
                SHIFT_PRESSED = false;
            }
            if let Some(character) = scan_code_to_ascii(scan_code, SHIFT_PRESSED) {
                utils::move_buffer_right(offset);
                offset = put_keyboard_input(character, offset);
            }
            else if scan_code == 0x0E{
                if offset > 0{
                    offset -= 2;
                    utils::move_buffer_left(offset);
                    set_cursor(offset);
                }
            }
            else if scan_code == 0x4B {
                if offset > 0{
                    offset -= 2;
                    set_cursor(offset);
                }
            }
            else if scan_code == 0x4D {
                if offset < utils::buffer_count(0) * 2{
                    offset += 2;
                    set_cursor(offset);
                }
            }
        }
    }
}