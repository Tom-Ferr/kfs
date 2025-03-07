use crate::io::outb;
use crate::idt::{install_irq_routine, IntReg};
use crate::get_reg;
use crate::procs::*;
use crate::queue::Queuable;
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
    let time = 10 * 18;
    unsafe{
        let target = TICKS + (time as usize);
        while TICKS < target {}
    }
}

pub unsafe fn switch_task(_regs: *mut IntReg){
    if let Some(ptr) = *CURRENT_TASK{
        let prev_task = ptr as *mut ProcessControlBlock;
        (*prev_task).set_regs(*_regs);
        (*prev_task).set_state(ProcStatus::Runnable);
        let prev_priority = (*prev_task).get_priority();
        QUEUES[prev_priority].roll();
        'priority_queue: for next_priority in 0..NUMBER_OF_QUEUES{
            if let Some(head) = QUEUES[next_priority].get(){
                let mut proc = head;
                while (*proc).get_state() != ProcStatus::Runnable{
                    proc = (*proc).get_next();
                    QUEUES[next_priority].roll();
                    if proc == head{
                        continue 'priority_queue;
                    }
                }
                (*(proc as *mut ProcessControlBlock)).change_process();
                (*(proc as *mut ProcessControlBlock)).change_context();

                *_regs = (*proc).get_regs();
                return;
            }
        }
    }
}

unsafe fn schedule(_regs: *const IntReg){
    if let Some(current_proc) = CURRENT_PROC {
        if let Some(signal) = (*current_proc).get_signal(){
            let handler = (*current_proc).get_handler(signal) as *const fn (i32);
            (*handler)(signal as i32);
        }
    }
    if TICKS % 10 == 0{
        switch_task(_regs as *mut IntReg);
    }
}