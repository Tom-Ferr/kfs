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
mod allocator;

use screens::render;
use gdt::init_gdt;
use paging::init_page_tables;
use multiboot::apply_mmap_info;
use allocator::block_pages;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
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
        let ke = &kernel_end as *const u32 as u32;
        
        init_page_tables();
        
        apply_mmap_info(multiboot_info + 0xC0000000);
        crate::allocator::block_pages(0x100000, (ke - 0xC0000000) - 0x100000);
    }
    
    // render();

    unsafe{
        
        let dir = crate::paging::DIR.as_mut().unwrap();

        let tab = &*(dir.get_page(0) as *const crate::paging::PageTable);
        tab.print_bitmap();

    }
    loop{}
    
}