use crate::io::*;
use crate::keyboard::KeyboardGuard;

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
                 .>......        ..>>...<<....>>.....>.<..>.";


pub static mut SCREEN1_BUFFER: ScreenBuff = ScreenBuff::new();
pub static mut SCREEN2_BUFFER: ScreenBuff = ScreenBuff::new();
pub static mut SCREEN3_BUFFER: ScreenBuff = ScreenBuff::new();
pub static mut CURRENT_SCREEN: *mut ScreenBuff = unsafe{&mut SCREEN1_BUFFER as *mut ScreenBuff};

#[derive(Copy, Clone)]
pub struct ScreenBuff{
    text: [u8; 25*80],
    color: [u8; 25*80],
    offset: u32,
}

impl ScreenBuff{
    const fn new() -> Self{
        unsafe{
            Self{
                text: [0; 25*80],
                color: [BACKGROUND_COLOR << 4 | TEXT_COLOR; 25*80],
                offset: 0,
            }
        }
    }

    pub fn import(&mut self){
        let vga_buffer = VGA_BUFFER as *const u8;
        unsafe{

            for i in 0..(25*80){
                self.text[i] = *vga_buffer.offset(i as isize * 2);
                self.color[i] = *vga_buffer.offset((i as isize * 2) + 1);
            }
        }
        self.offset = get_cursor();
    }

    pub fn export(&self){
        let vga_buffer = VGA_BUFFER as *mut u8;
        unsafe{

            for i in 0..(25*80){
                *vga_buffer.offset(i as isize * 2) = self.text[i];
                *vga_buffer.offset((i as isize * 2) + 1) = self.color[i];
            }
        }
        set_cursor(self.offset);
    }

    pub fn update_color_mode(&mut self, index: usize, prev_text_color: u8){
        unsafe{
            let current: u8 = self.color[index] & 0b1111;
            let mut color = TEXT_COLOR;
            if current != prev_text_color {
                color = current;
            }
            self.color[index] = BACKGROUND_COLOR << 4 | color;
        }
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
               
pub fn welcome_screen() {
    let guard = KeyboardGuard::new();
    clear_vga();
    
    enable_cursor(false);

    put_vga_string(ASCII_ART);


    crate::timer::sleep(10);

    clear_vga();
    enable_cursor(true);
}