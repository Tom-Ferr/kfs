use crate::io::*;
use crate::key_handlers::*;
use crate::printf;
use crate::keyboard::SHIFT_PRESSED;

#[allow(unused_imports)]
use crate::printk;

// static ASCII_ART: &[u8] = b"
//            +++++[>++[>+>+        ++>++++>++++>++++>++++++
//           >++++++>+++++++        ++>+++++++++<<<<<<<<<-]>>
//          >+>+>+> >>>+[<]<        -]>>       >++>-->>+>>++>+
//         >--<<<<  <<<.....         .>            ....<......
//        ...>...   <<.>....                       >.>>>>>.<.
//        <<<<..     ..<....                      >..>>>>>.<
//       .<<<<.      >>>.<<.                     >>>>>.<.<
//       <<<<<       <.>...>                    >>>.>>>.
//      <<<.<        <<<..>>                  .>>>>>.<
//     <.<<<         <<...>>                 >>>.<<<
//    <..<.          ...>...               <<.>..>.
//    >>.<.<<...>>...<<...>>...<         <....>>..
//   .<<<.>.>>..>.<<.......<....        .....>...
//                  <<.>...            .....>...
//                  <......           .>>>.<<..
//                  <<.>...          .....>...<......>.>>.<.<<<
//                  .>......        ..>>...<<....>>.....>.<..>.";


static mut SCREEN1_BUFFER: ScreenBuff = ScreenBuff::new();
static mut SCREEN2_BUFFER: ScreenBuff = ScreenBuff::new();
static mut SCREEN3_BUFFER: ScreenBuff = ScreenBuff::new();
pub static mut CURRENT_SCREEN: *mut ScreenBuff = unsafe{&mut SCREEN1_BUFFER as *mut ScreenBuff};

#[derive(Copy, Clone)]
pub struct ScreenBuff{
    text: [u8; 150*80],
    color: [u8; 150*80],
    offset: u32,
}

impl ScreenBuff{
    const fn new() -> Self{
        unsafe{
            Self{
                text: [0; 150*80],
                color: [BACKGROUND_COLOR << 4 | TEXT_COLOR; 150*80],
                offset: 0,
            }
        }
    }

    pub fn import(&mut self){
        let vga_buffer = VGA_BUFFER as *const u8;
        unsafe{

            for i in 0..(150*80){
                self.text[i] = *vga_buffer.offset(i as isize * 2);
                self.color[i] = *vga_buffer.offset((i as isize * 2) + 1);
            }
        }
        self.offset = get_cursor();
    }

    pub fn export(&self){
        let vga_buffer = VGA_BUFFER as *mut u8;
        unsafe{

            for i in 0..(150*80){
                *vga_buffer.offset(i as isize * 2) = self.text[i];
                *vga_buffer.offset((i as isize * 2) + 1) = self.color[i];
            }
        }
        set_cursor(self.offset);
    }
}

#[derive(PartialEq, Clone)]
pub enum Screen {
    Screen1,
    Screen2,
    Screen3,
}

pub unsafe fn change_screen(screen: Screen){
    (*CURRENT_SCREEN).import();
    match screen{
        Screen::Screen1 => {CURRENT_SCREEN = &mut SCREEN1_BUFFER as *mut ScreenBuff},
        Screen::Screen2 => {CURRENT_SCREEN = &mut SCREEN2_BUFFER as *mut ScreenBuff},
        Screen::Screen3 => {CURRENT_SCREEN = &mut SCREEN3_BUFFER as *mut ScreenBuff},
        _ => {}
    }
    (*CURRENT_SCREEN).export();
}

// impl Screen {
//     pub fn get_function(&self) -> Option<fn(& Screen) -> Screen> {
//         match self {
//             Screen::Screen1 => Some(screen_1),
//             Screen::Screen2 => Some(screen_2),
//             Screen::Screen3 => Some(screen_3),
//         }
//     }
// }


// pub fn render() -> ! {

//     let mut current_screen: Screen = Screen::Screen1;
//     loop{
//         if let Some(run) = current_screen.get_function(){
//             current_screen = run(&current_screen);
                
//         }
//     }
// }
               
// fn screen_1(current_screen: &Screen) -> Screen {
//     clear_vga();
    
//     enable_cursor(false);

//     put_vga_string(ASCII_ART);

//     printf!("\n\n\n\n              Please, press \'CTRL + {}\' to check bonuses", "(2 or 3)");


//     loop {
//         let scan_code = read_key();
//         match scan_code {
//             // 0x1D => { if let Some(f) = handle_shortcuts(current_screen){ return f;} },
//             _ => {}
//         }
//     }
// }

// fn screen_2(current_screen: &Screen) -> Screen {
//     enable_cursor(true);
//     unsafe {
//         #[allow(static_mut_refs)]
//         SCREEN2_BUFFER.export();
    
//         loop {
//             let mut offset = get_cursor();
//             let scan_code = read_key();
//             match scan_code{
//                 0x0E => handle_backspace(&mut offset),
//                 0x4B => handle_left_arrow(&mut offset),
//                 0x4D => handle_right_arrow(&mut offset),
//                 0x53 => handle_delete(&mut offset),
//                 0x1C => handle_enter(&mut offset),
//                 0x1D => { if let Some(f) = handle_shortcuts(current_screen){ #[allow(static_mut_refs)]SCREEN2_BUFFER.import(); return f;} },
//                 _ => handle_character(scan_code, SHIFT_PRESSED, &mut offset),
//             }
            
//         }
//     }
// }

// fn screen_3(current_screen: &Screen) -> Screen{
//     clear_vga();
    
//     enable_cursor(false);

//     printk!(INFO, "This is an example of {} log message\n", "INFO");
//     printk!(WARNING, "This is an example of {} log message\n", "WARNING");
//     printk!(ERROR, "This is an example of {} log message\n", "ERROR");
//     printk!(DEBUG, "This is an example of {} log message\n", "DEBUG");

//     loop {
//         let scan_code = read_key();
//         match scan_code {
//             0x1D => { if let Some(f) = handle_shortcuts(current_screen){ return f;} },
//             _ => {}
//         }
//     }
// }