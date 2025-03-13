use crate::idt::IntReg;
use crate::procs::*;
use core::arch::asm;

pub unsafe fn write(ptr: *const u8, len: u32){
    asm!("
        mov eax, 2
        int 0x80
    ", in("ebx")ptr, in("ecx")len);
}

pub unsafe fn fork() -> i32{
    let ret: i32;
    asm!("
        mov eax, 3
        int 0x80
    ", out("eax")ret);
    ret
}

pub unsafe fn exit(exit_code: i32){
    asm!("
        mov ebx, {}
        mov eax, 4
        int 0x80
    ", in(reg)exit_code);
}

pub unsafe fn get_pid() -> u32{
    let ret: u32;
    asm!("
        mov eax, 5
        int 0x80
    ", out("eax")ret);
    ret
}

pub unsafe fn wait(wstatus: &mut i32) -> i32{
    let ptr = wstatus as *mut i32;
    let ret: i32;
    asm!("
        mov ebx, {}
        mov eax, 6
        int 0x80
    ", in(reg)ptr, out("eax")ret);
    ret
}

pub fn syscall_handler(regs: *mut IntReg){
    
    let eax: u32 = unsafe {(*regs).get_eax()};
    let ebx: u32 = unsafe {(*regs).get_ebx()};
    let ecx: u32 = unsafe {(*regs).get_ecx()};

    match eax{
        1 => unsafe {crate::io::_print_fmt_str(*(ebx as *const core::fmt::Arguments))},
        2 => unsafe {crate::io::put_vga_ptr(ebx as *const u8, ecx)},
        3 => unsafe {if let Some(child) = sys_fork(){ (*regs).set_eax(child) } else{(*regs).set_eax(u32::MAX)}},
        4 => unsafe {sys_exit(ebx as i32)},
        5 => unsafe {if let Some(proc) = *CURRENT_TASK {let pid = (*proc).get_pid(); (*regs).set_eax(pid)}},
        6 => unsafe {if let Some(pid) = sys_wait(ebx as *mut i32){ (*regs).set_eax(pid) } else{(*regs).set_eax(u32::MAX)}},
        _ => {crate::printk!(INFO, "Nope")},
    }
}