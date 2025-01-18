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

use screens::render;
use gdt::init_gdt;
use paging::init_page_tables;
use multiboot::apply_mmap_info;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        printk!(ERROR, "{}", location);
    }
    loop {}
}

extern "C" {
    static kernel_start: u32;
    static kernel_end: u32;
}

#[no_mangle]
pub extern "C" fn kernel(multiboot_info: u32) -> ! {

    init_gdt();

    unsafe{
        let ks = &kernel_start as *const u32 as u32;
        let ke = &kernel_end as *const u32 as u32 - 0xC0000000;
        
        init_page_tables();
        apply_mmap_info(multiboot_info + 0xC0000000, ks, ke);
    }
    
    // render();

    unsafe{
        
        let dir = crate::paging::DIR.as_mut().unwrap();

        let tab = &*(dir.get_page(0) as *const crate::paging::PageTable);
        tab.print_bitmap();

    }
    loop{}
    
}