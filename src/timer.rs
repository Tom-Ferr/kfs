use crate::io::outb;
use crate::idt::{install_irq_routine, IntReg};
use crate::procs::*;
use crate::queue::Queuable;

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

pub fn sleep(unit: u32){
    let time = unit * 18;
    unsafe{
        let target = TICKS + (time as usize);
        while TICKS < target {}
    }
}

pub unsafe fn switch_task(_regs: *const IntReg) -> Option<*mut ProcessControlBlock>{
    if let Some(ptr) = *CURRENT_TASK{
        let prev_task = ptr as *mut ProcessControlBlock;
        (*prev_task).set_regs(*_regs);
        if (*prev_task).get_state() == ProcStatus::Running{
            (*prev_task).set_state(ProcStatus::Runnable);
        }
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
                return Some(proc as *mut ProcessControlBlock);
            }
        }
    }
    None
}

unsafe fn schedule(_regs: *const IntReg){
    if let Some(current_proc) = CURRENT_PROC {
        if let Some(signal) = (*current_proc).get_signal(){
            let handler = (*current_proc).get_handler(signal) as *const fn (i32);
            (*handler)(signal as i32);
        }
    }
    if TICKS % 10 == 0{
        if let Some(proc) = switch_task(_regs){

            (*proc).change_process();
            (*proc).change_context();
            
            *(_regs as *mut IntReg) = (*proc).get_regs();
            
        }
    }
}