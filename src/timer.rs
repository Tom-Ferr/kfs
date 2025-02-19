use crate::io::outb;
use crate::idt::{install_irq_routine, IntReg};
use crate::get_reg;
use crate::procs::*;
use core::arch::asm;

static mut TICKS: usize = 0;
const FREQ: u32 = 100;

fn timer(_regs: *const IntReg){
    unsafe{
        schedule(_regs);
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

unsafe fn switch_task(){
    if let Some(_) = *CURRENT_TASK{
        let current_task = *(*CURRENT_TASK).as_mut().unwrap() as *mut ProcessControlBlock;
        (*current_task).set_esp(get_reg!(esp) as u32);
        let current_priority = (*current_task).get_priority();
        QUEUES[current_priority].roll();
        for next_priority in 0..NUMBER_OF_QUEUES{
            if let Some(head) = QUEUES[next_priority].get(){
                CURRENT_PROC = Some(head);
                let next_task = *(*CURRENT_TASK).as_mut().unwrap() as *mut ProcessControlBlock;
                let dir = (*next_task).get_dir() - 0xC0000000;
                asm!("mov cr3, {}", in(reg) dir);
                asm!("mov esp, {}", in(reg)(*next_task).get_esp());
                break;
            }
        }
    }
}

#[allow(dead_code)]
fn schedule(_regs: *const IntReg){
    unsafe{
        if TICKS % 10 == 0{
            switch_task();
        }
    }
}