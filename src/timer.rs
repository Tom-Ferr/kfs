use crate::io::outb;
use crate::idt::{install_irq_routine, IntReg};
use crate::get_reg;
use crate::procs::*;
use core::arch::asm;

static mut TICKS: usize = 0;
const FREQ: u32 = 100;

fn timer(_regs: *const IntReg){
    unsafe{
        // schedule(_regs);
        TICKS += 1;
    }
}

pub fn init_timer(){
    install_irq_routine(0, timer);

    let divisor: u32 = 1193182 / FREQ;
    unsafe{
        outb(0x43, 0x36);
        outb(0x40, (divisor & 0xFF) as u8);
        outb(0x40, ((divisor >> 8) & 0xFF) as u8);
    }
}

pub fn sleep(){
    let time = 18 * 60;
    unsafe{
        let target = TICKS + (time as usize);
        while TICKS < target {}
    }
}

#[allow(dead_code)]
fn schedule(_regs: *const IntReg){
    unsafe{
        if TICKS % 10 == 0{
            {
                let current_task = *(*CURRENT_TASK).as_mut().unwrap() as *mut ProcessControlBlock;
                (*current_task).set_esp(get_reg!(esp) as u32);
                QUEUES[0].roll();
            }
            let head = QUEUES[0].get();
            CURRENT_PROC = head;
            let current_task = *(*CURRENT_TASK).as_mut().unwrap() as *mut ProcessControlBlock;
            asm!("mov esp, {}", in(reg)(*current_task).get_esp());
            
        }
    }
}