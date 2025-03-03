use crate::get_reg;
use crate::procs::{CURRENT_TASK, ProcessControlBlock};

pub static mut  _TSS: TSS = TSS::new();

#[repr(C, packed)]
pub struct TSS {
	prev_tss: u32,
	esp0: u32,
	ss0: u32,
	esp1: u32,
	ss1: u32,
	esp2: u32,
	ss2: u32,
	cr3: u32,
	eip: u32,
	eflags: u32,
	eax: u32,
	ecx: u32,
	edx: u32,
	ebx: u32,
	esp: u32,
	ebp: u32,
	esi: u32,
	edi: u32,
	es: u32,
	cs: u32,
	ss: u32,
	ds: u32,
	fs: u32,
	gs: u32,
	ldt: u32,
	trap: u16,
	iomap: u16,
}

impl TSS{
	const fn new() -> Self {
		Self {
			prev_tss: 0,
			esp0: 0,
			ss0: 0,
			esp1: 0,
			ss1: 0,
			esp2: 0,
			ss2: 0,
			cr3: 0,
			eip: 0,
			eflags: 0,
			eax: 0,
			ecx: 0,
			edx: 0,
			ebx: 0,
			esp: 0,
			ebp: 0,
			esi: 0,
			edi: 0,
			es: 0,
			cs: 0,
			ss: 0,
			ds: 0,
			fs: 0,
			gs: 0,
			ldt: 0,
			trap: 0,
			iomap: 0,
		}
	}
	
    pub fn init(&mut self) {

        self.ss0 = 0x18;
        self.esp0 = unsafe {get_reg!(esp)} as u32;

        self.iomap = size_of::<TSS>() as u16;

        self.cs = 0x0b;
        self.ss = 0x1b;
        self.es = 0x13;
        self.ds = 0x13;
        self.fs = 0x13;
        self.gs = 0x13;
    }

	pub fn as_ref(&self) -> &Self {
		self
	}

	fn set_ss0(&mut self, kernel_ss: u32){
		self.ss0 = kernel_ss;
	}

	fn set_esp0(&mut self, kernel_esp: u32){
		self.esp0 = kernel_esp;
	}

	pub fn set_stack(&mut self, kernel_ss: u32, kernel_esp: u32){
		self.set_ss0(kernel_ss);
		self.set_esp0(kernel_esp);
	}
}

#[no_mangle]
pub extern "C" fn switch_tss(){
	unsafe{
		if let Some(current_task) = *CURRENT_TASK{
			_TSS.set_stack((*current_task).get_kernel_ss(), (*current_task).get_kernel_esp());
		}
	}
}