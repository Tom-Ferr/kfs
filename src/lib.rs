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
            else if scan_code == 0x2A + 0x80 || scan_code == 0x36 + 0x80{
                SHIFT_PRESSED &= 0b10;
            }
            else if scan_code == 0x3A{
                SHIFT_PRESSED ^= 0b10;
            }
            if let Some(character) = scan_code_to_ascii(scan_code, SHIFT_PRESSED) {
                utils::move_buffer_right(offset);
                offset = put_keyboard_input(character, offset);
            }
            else if scan_code == 0x0E{ //bacspace
                if offset > 0{
                    offset -= 2;
                    utils::move_buffer_left(offset);
                    set_cursor(offset);
                }
            }
            else if scan_code == 0x4B { // left arrow
                if offset > 0{
                    offset -= 2;
                    set_cursor(offset);
                }
            }
            else if scan_code == 0x4D { // right arrow
                if offset < utils::buffer_count(0) * 2{
                    offset += 2;
                    set_cursor(offset);
                }
            }
            else if scan_code == 0x53 { // delete
                if offset < utils::buffer_count(0) * 2{
                    utils::move_buffer_left(offset);
                }
            }
            else if scan_code == 0x1C { //enter
                offset = move_offset_to_new_line(offset);
                if offset >= 25 * 80 * 2 {
                    offset = scroll_ln(offset);
                }
                set_cursor(offset);
            }
        }
    }
}