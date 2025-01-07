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

    init_page_tables();

    // render();

    unsafe{

        let dir = crate::get_reg!(cr3);

        let tab = (*dir & 0xfffff000) as *const u32;
        
        io::memory_dump(tab, tab.wrapping_add(1024));
        crate::printk!(DEBUG, "Page Table Address = {:?}\n", &paging::TABLE.tables.data as *const [u32; 1024]);

        crate::printk!(DEBUG, "Page Directory First Entry = {:x}\n", paging::DIR.tables.data[0]);

        crate::printk!(DEBUG, "Page Directory First Entry (Filtered) = {:x}\n", (paging::DIR.tables.data[0] & 0xfffff000));

        crate::printk!(DEBUG, "Value of Filtred Address = {:x}\n", *((paging::DIR.tables.data[0] & 0x3ff000) as *const u32));

        crate::printk!(DEBUG, "Page Table First Entry = {:x}\n", paging::TABLE.tables.data[0]);
    }
    loop{}
    
}