use crate::io::*;
use crate::key_handlers::*;

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

                 

                  Please, press \'CTRL + 2\' to check bonuses
               ";
static mut SHIFT_PRESSED: u8 = 0b0;

const L_SHIFT: u8 = 0x2A;
const R_SHIFT: u8 = 0x36;
const L_SHIFT_RELEASE: u8 = 0x2A + 0x80;
const R_SHIFT_RELEASE: u8 = 0x36 + 0x80;
const CAPS_LOCK:u8 = 0x3A;

#[derive(PartialEq, Clone)]
pub enum Screen {
    Screen1,
    Screen2,
}

impl Screen {
    pub fn get_function(&self) -> Option<fn(& Screen) -> Screen> {
        match self {
            Screen::Screen1 => Some(screen_1),
            Screen::Screen2 => Some(screen_2),
        }
    }
}


pub fn render() -> ! {

    let mut current_screen: Screen = Screen::Screen1;
    loop{
        if let Some(run) = current_screen.get_function(){
            current_screen = run(&current_screen);
                
        }
    }
}
               
pub fn screen_1(current_screen: &Screen) -> Screen {
    clear_vga();
    
    enable_cursor(false);

    let offset = put_vga_string(ASCII_ART, 0);

    set_cursor(offset);

    loop {
        let scan_code = read_key();
        match scan_code {
            0x1D => { if let Some(f) = handle_shortcuts(current_screen){ return f;} },
            _ => {}
        }
    }
}

pub fn screen_2(current_screen: &Screen) -> Screen {
    enable_cursor(true);
    clear_vga();
    
    loop {
        let mut offset = get_cursor();
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
                0x1D => { if let Some(f) = handle_shortcuts(current_screen){ return f;} },
                _ => handle_character(scan_code, SHIFT_PRESSED, &mut offset),
            }
            
        }
    }
}