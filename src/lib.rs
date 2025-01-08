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

use screens::render;
use gdt::init_gdt;
use paging::init_page_tables;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel() -> ! {

    init_gdt();

    // init_page_tables();

    render();

    unsafe{

        // let dir = (crate::get_reg!(cr3) as u32 + 0xC0000000) as *const u32;

        // let virtual_mem: u32 = 0xC0000800;

        // let offset = virtual_mem & 0xfff;

        // let ti = (virtual_mem >> 12) & 0x3ff;

        // let di = (virtual_mem >> 22) & 0x3ff;

        
        // io::memory_dump(dir, dir.wrapping_add(1024));

        // crate::printk!(DEBUG, "offset = {:?}\n", offset);
        // crate::printk!(DEBUG, "table index = {:?}\n", ti);
        // crate::printk!(DEBUG, "directory index = {:?}\n", di);

        // crate::printk!(DEBUG, "directory addr = {:?}\n", dir);
        // crate::printk!(DEBUG, "directory addr = {:x}\n", dir as u32);

        // let entry = dir.wrapping_add(di as usize) as *const u32;
        // crate::printk!(DEBUG, "Directory Table 768th entry = {:?}\n", entry);

        // let filtered = (entry as u32 & 0xfffff000) as *const u32;
        // crate::printk!(DEBUG, "Directory Table 768th entry = {:?}\n", filtered);

        // let tab = *filtered as u32;
        // crate::printk!(DEBUG, "Page Table 1st entry = {:?}\n", tab);

        // let target = *filtered.wrapping_add(ti as usize);
        // crate::printk!(DEBUG, "Page Table 256th entry = 0x{:x}\n", target);

        
        
        
        
        
        // crate::printk!(DEBUG, "Page Table Address = {:?}\n", &paging::TABLE.tables.data as *const [u32; 1024]);

        // crate::printk!(DEBUG, "Page Directory First Entry = {:x}\n", paging::DIR.tables.data[0]);

        // crate::printk!(DEBUG, "Page Directory First Entry (Filtered) = {:x}\n", (paging::DIR.tables.data[0] & 0xfffff000));

        // crate::printk!(DEBUG, "Value of Filtred Address = {:x}\n", *((paging::DIR.tables.data[0] & 0x3ff000) as *const u32));

        // crate::printk!(DEBUG, "Page Table First Entry = {:x}\n", paging::TABLE.tables.data[0]);
    }
    loop{}
    
}