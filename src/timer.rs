use crate::io::outb;
use crate::idt::{install_irq_routine, IntReg};
use core::arch::asm;

static mut TICKS: usize = 0;
const FREQ: u32 = 100;

fn timer(_regs: *const IntReg){
    unsafe{
        TICKS += 1;
        // schedule();
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
fn schedule(){
    unsafe{
        if TICKS % (18 * 60) == 0{
            asm!("
            mov eax, 1
            int 0x80
            ")
        }
    }
}