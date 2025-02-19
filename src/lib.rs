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

// use screens::render;
use gdt::init_gdt;
use paging::init_page_tables;
use multiboot::apply_mmap_info;
use idt::init_idt;
use timer::init_timer;
use keyboard::init_keyboard;
use crate::procs::exec_fn;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        unsafe{
            let cr2 = get_reg!(cr2) as u32;
            printk!(ERROR, "{}, {}, {:x}", location, _info.message(), cr2);
        }
    }
    unsafe { panic_halt(); }
}

extern "C" {
    static kernel_start: u32;
    static kernel_end: u32;
    fn panic_halt() -> !;
}

#[no_mangle]
fn test() -> ! {
    // printf!("Welcome to User Land");
    // let string: &[u8] = b"Hello World";
    // crate::io::call_write(string);
    loop{}
}

#[no_mangle]
pub extern "C" fn kernel(multiboot_info: u32) -> ! {

    init_gdt();
    init_idt();
    init_timer();
    init_keyboard();

    crate::io::clear_vga();

    unsafe{
        let ks = &kernel_start as *const u32 as u32;
        let ke = &kernel_end as *const u32 as u32 - 0xC0000000;
        
        init_page_tables();
        if let Err(..) = apply_mmap_info(multiboot_info + 0xC0000000, ks, ke){
            panic!();
        }

        exec_fn(0xC0000000, test as u32, ke);
    }
    loop{}
}