use crate::idt::IntReg;

pub fn syscall_handler(regs: *const IntReg){
    
    let eax: u32 = unsafe {(*regs).get_eax()};

    match eax{
        1 => crate::printk!(INFO, "syscall triggered"),
        _ => crate::printk!(INFO, "Nope"),
    }
}