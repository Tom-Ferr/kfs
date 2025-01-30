use crate::io::outb;
use crate::idt::{install_irq_routine, IntReg};

static mut TICKS: usize = 0;
const FREQ: u32 = 100;

fn timer(regs: *const IntReg){
    unsafe{
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