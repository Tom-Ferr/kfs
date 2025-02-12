use crate::idt::IntReg;

pub fn syscall_handler(regs: *const IntReg){
    
    let eax: u32 = unsafe {(*regs).get_eax()};
    let ebx: u32 = unsafe {(*regs).get_ebx()};
    let ecx: u32 = unsafe {(*regs).get_ecx()};

    match eax{
        1 => unsafe {crate::io::_print_fmt_str(*(ebx as *const core::fmt::Arguments))},
        _ => crate::printk!(INFO, "Nope"),
    }
}