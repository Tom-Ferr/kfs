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
mod message;
mod ext2;
mod vfs;
mod ide;

use crate::screens::welcome_screen;
use gdt::init_gdt;
use paging::init_page_tables;
use multiboot::apply_mmap_info;
use idt::init_idt;
use timer::init_timer;
use keyboard::init_keyboard;
use crate::procs::{exec_fn, load_process};
use crate::syscalls::{fork, wait};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
            printk!(ERROR, "{}, {}", location, _info.message());
    }
    unsafe { panic_halt(); }
}

extern "C" {
    static kernel_start: u32;
    static kernel_end: u32;
    fn panic_halt() -> !;
}

unsafe fn initial_task() -> ! {
    let pid = fork();
    if pid == 0{
        user_land();
    }
    let mut status: i32 = 42;
    loop{
        wait(&mut status);
    }
}

unsafe fn user_land() -> ! {
    loop {
        printf!("user_land_pid_{}\n", crate::syscalls::get_pid());
        crate::timer::sleep(15);
    }
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
        
    let _ = load_process(0xC0000000, ke + 0xC0000000);
        
    init_idt();
    init_timer();
    init_keyboard();
        
    welcome_screen();

    let _ = exec_fn(0xC0000000, initial_task as u32, ke);

    panic!();
}