use core::mem::size_of;
use core::arch::asm;

use crate::io::outb;
use crate::syscalls::syscall_handler;
use crate::get_reg;
use crate::procs::{CURRENT_TASK, ProcessControlBlock};

#[allow(dead_code)]
extern "C" {
    fn idt_flush(idt_ptr: *const Idtr);

    fn clean_registers();

    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();

    fn isr128();
    fn isr177();

    fn irq0();
    fn irq1();
    fn irq2();
    fn irq3();
    fn irq4();
    fn irq5();
    fn irq6();
    fn irq7();
    fn irq8();
    fn irq9();
    fn irq10();
    fn irq11();
    fn irq12();
    fn irq13();
    fn irq14();
    fn irq15();
}

static ISR: [unsafe extern "C" fn(); 32] = [isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7,
                                            isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15,
                                            isr16, isr17, isr18, isr19, isr20, isr21, isr22,isr23,
                                            isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31];

static IRQ: [unsafe extern "C" fn(); 16] = [irq0, irq1, irq2, irq3, irq4, irq5, irq6, irq7,
                                            irq8, irq9, irq10, irq11, irq12, irq13, irq14, irq15];

static mut IRQ_ROUTINES: [Option<fn(*const IntReg)>; 16] = [None; 16];

static EXCEPT_MSG: [&str; 32] = [
    "Division by Zero", 
    "Debug", 
    "Non Maskable Interrupt", 
    "Breakpoint", 
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Coprocessor not available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid Task State Segment",
    "Segment not present",
    "Stack Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved"
];

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct IntReg{
    cr2: u32,
    ds: u32,
    edi: u32,
    esi: u32,
    ebp: u32,
    esp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    int_no: u32,
    err_code: u32,
    eip: u32,
    cs: u32,
    eflags: u32,
    useresp: u32,
    ss: u32,
}

impl IntReg{

    pub fn set_eax(&mut self, value: u32) {
        self.eax = value;
    }

    pub fn set_eip(&mut self, value: u32) {
        self.eip = value;
    }

    pub fn get_eax(&self) -> u32 {
        self.eax
    }

    pub fn get_ebx(&self) -> u32 {
        self.ebx
    }

    pub fn get_ecx(&self) -> u32 {
        self.ecx
    }

    pub fn get_edx(&self) -> u32 {
        self.edx
    }

    pub fn get_esi(&self) -> u32 {
        self.esi
    }

    pub fn get_edi(&self) -> u32 {
        self.edi
    }

    pub fn get_eip(&self) -> u32 {
        self.eip
    }
}

pub struct InterruptGuard {}

impl InterruptGuard{
    pub fn new() -> Self{
        InterruptGuard::clear();
        Self{}
    }
    fn clear(){
        unsafe{asm!("cli");}
    }
    #[allow(dead_code)]
    fn set(){
        unsafe{asm!("sti");}
    }

}

impl Drop for InterruptGuard{
    fn drop(&mut self){
        init_idt();
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
struct IdtDescriptor{
    base_low: u16,
    selector: u16,
    zero: u8,
    flags: u8,
    base_high: u16,
}

impl IdtDescriptor {
    fn new(base: u32, selector: u16, flags: u8) -> Self {
        Self{
            base_low: (base & 0xFFFF) as u16,
            selector: selector,
            zero: 0,
            flags: flags | 0x60,
            base_high: (base >> 16) as u16,
        }
    }
}

#[repr(C, align(0x10))]
struct IdtEntries{
    entries: [IdtDescriptor; 256],
}

impl IdtEntries {
    fn new() -> Self {
        let mut ret = Self{entries: [IdtDescriptor::default(); 256]};
        for i in 0..32{
            ret.entries[i] = IdtDescriptor::new(ISR[i as usize] as *const () as u32, 0x08, 0x8E);
        }

        ret.entries[128] = IdtDescriptor::new(isr128 as *const () as u32, 0x08, 0x8E);
        ret.entries[177] = IdtDescriptor::new(isr177 as *const () as u32, 0x08, 0x8E);

        for i in 32..=47{
            ret.entries[i] = IdtDescriptor::new(IRQ[i-32 as usize] as *const () as u32, 0x08, 0x8E);
        }
        ret
    }

}

#[repr(C, packed)]
struct Idtr{
    limit: u16,
    base: *const IdtEntries,
}

impl Idtr {
    fn new(entries: *const IdtEntries) -> Self {
        Self{
            limit: (size_of::<IdtEntries>() - 1) as u16,
            base: entries,
        }
    }
}

pub fn install_irq_routine(index: usize, handler: fn(*const IntReg)){
    unsafe{
        IRQ_ROUTINES[index] = Some(handler);
    }
}

#[allow(dead_code)]
pub fn uninstall_irq_routine(index: usize){
    unsafe{
        IRQ_ROUTINES[index] = None;
    }
}

#[no_mangle]
pub unsafe extern "C" fn irq_handler(regs: *const IntReg){
    if let Some(task) = *CURRENT_TASK{
        (*task).export_ebp();
    }
    if let Some(handler) = IRQ_ROUTINES[((*regs).int_no - 32) as usize]{
        handler(regs);
    }
    if (*regs).int_no >= 40{
        outb(0xA0, 0x20);
    }
    outb(0x20, 0x20);
}

#[no_mangle]
pub unsafe extern "C" fn isr_handler(regs: *mut IntReg){
    if let Some(task) = *CURRENT_TASK{
        (*(task as *mut ProcessControlBlock)).set_regs(*regs);
        let err_code = (*regs).err_code;
        match (*regs).int_no {
                
            0..32 => panic!("{}, error code: {}, pid: {}, eip: {:#x}", EXCEPT_MSG[(*regs).int_no as usize], err_code, (*task).get_pid(), (*regs).get_eip()),
            0x80 => syscall_handler(regs),
            _   => {},
        }
    }
}

pub fn init_idt() {
    unsafe{
        outb(0x20, 0x11);
        outb(0xA0, 0x11);

        outb(0x21, 0x20);
        outb(0xA1, 0x28);

        outb(0x21, 0x04);
        outb(0xA1, 0x02);

        outb(0x21, 0x01);
        outb(0xA1, 0x01);

        outb(0x21, 0x0);
        outb(0xA1, 0x0);

        let idt_entries = IdtEntries::new();
        let idtr = Idtr::new(&idt_entries);

        idt_flush(&idtr);
    }
}