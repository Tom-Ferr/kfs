#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
mod utils;
mod key_handlers;
mod commands;
mod screens;

use screens::render;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel() -> ! {

    render();
    
}