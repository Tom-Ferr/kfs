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

use screens::render;
use gdt::init_gdt;
use paging::init_page_tables;
use multiboot::read_multiboot_info;

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

    unsafe{

        let ks = &kernel_start as *const u32 as usize;
        let ke = &kernel_end as *const u32 as usize;

        crate::printf!("kernel start = 0x{:x}\n", ks);
        crate::printf!("kernel end = 0x{:x}\n", ke);
    }

    read_multiboot_info(multiboot_info + 0xC0000000);

    // init_gdt();

    // init_page_tables();

    // render();

    // unsafe{
        
    //     let dir = crate::paging::DIR.as_mut().unwrap();

    //     let virtual_mem: usize = 0xC03FF000;

    //     let offset: usize = virtual_mem & 0xfff;

    //     let ti: usize = (virtual_mem >> 12) & 0x3ff;

    //     let di: usize = (virtual_mem >> 22) & 0x3ff;

        
        // io::memory_dump(dir, dir.wrapping_add(1024));

    //     crate::printk!(DEBUG, "offset = {:?}\n", offset);
    //     crate::printk!(DEBUG, "table index = {:?}\n", ti);
    //     crate::printk!(DEBUG, "directory index = {:?}\n", di);

    //     if let Ok(..) = dir.new_page(){
    //         let entry = (*dir.directory).data[769] as *const u32;
    //         crate::printk!(DEBUG, "Directory Table 769th entry = {:?}\n", entry);

    //         let filtered = (entry as u32 & 0xfffff000) as *const u32;
    //         crate::printk!(DEBUG, "Filtered Directory Table 769th entry = {:?}\n", filtered);

    //         let tab = *((filtered as u32) as *const u32);
    //         crate::printk!(DEBUG, "Page Table 1st entry = 0x{:x}\n", tab);

    //         let target = *((filtered.wrapping_add(1023) as u32) as *const u32);
    //         crate::printk!(DEBUG, "Page Table {}th entry = 0x{:x}\n", 1023, target);

    //         crate::printk!(DEBUG, "Dir Size {}\n", dir.size);
    //     }
    //     else{
    //         crate::printk!(DEBUG, "Error\n");
    //     }


    // }
    loop{}
    
}