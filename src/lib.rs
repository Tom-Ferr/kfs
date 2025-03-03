#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
mod utils;
mod key_handlers;
mod commands;
mod screens;
mod gdt;
mod paging;
mod multiboot;
mod malloc;
mod idt;
mod timer;
mod keyboard;
mod syscalls;
mod tss;
mod procs;
mod queue;
mod signals;

use crate::screens::welcome_screen;
use gdt::init_gdt;
use paging::init_page_tables;
use multiboot::apply_mmap_info;
use idt::init_idt;
use timer::init_timer;
use keyboard::init_keyboard;
use crate::procs::{exec_fn, load_process};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        unsafe{
            let cr2 = get_reg!(cr2) as u32;
            printk!(ERROR, "{}, {}, {:#x}", location, _info.message(), cr2);
        }
    }
    unsafe { panic_halt(); }
}

extern "C" {
    static kernel_start: u32;
    static kernel_end: u32;
    fn panic_halt() -> !;
}

fn user_land() -> ! {
    loop{}
}

#[no_mangle]
pub unsafe extern "C" fn kernel(multiboot_info: u32) -> ! {

    let ks = &kernel_start as *const u32 as u32;
    let ke = &kernel_end as *const u32 as u32 - 0xC0000000;

    init_gdt();
        
    init_page_tables();
    if let Err(..) = apply_mmap_info(multiboot_info + 0xC0000000, ks, ke){
        panic!();
    }
        
    load_process(0xC0000000, ke + 0xC0000000);
        
    init_idt();
    init_timer();
    init_keyboard();
        
    welcome_screen();

    exec_fn(0xC0000000, user_land as u32, ke);
        
    panic!();
}