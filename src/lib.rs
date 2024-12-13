#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// static ASCII_ART: &[u8] = b"
//          :::       ::::::::   
//        :+:       :+:    :+:   
//      +:+ +:+          +:+     
//    +#+  +:+        +#+        
//  +#+#+#+#+#+    +#+           
//       #+#     #+#             
//      ###    ########            
// ";

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
";

#[no_mangle]
pub extern "C" fn kernel() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    let mut row = 0;
    let mut col = 0;

    for &byte in ASCII_ART {
        unsafe {
            if byte == b'\n' {
                row += 1;
                col = 0;
                continue;
            }
            let offset = (row * 80 + col) * 2; // VGA buffer is 80 columns wide
            *vga_buffer.offset(offset as isize) = byte;
            *vga_buffer.offset(offset as isize + 1) = 0xa; // Light green text on black
            col += 1;
        }
    }

    loop {}
}
