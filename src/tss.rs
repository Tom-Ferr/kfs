
#[derive(Copy, Clone, Default)]
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
    pub const new() -> Self {
        let ret = TSS::default();

        ret.ss0 = 0x18;
        ret.esp0 = unsafe {get_reg!(esp)};

        ret.iomap = size_of::<TSS>();

        ret.cs = 0x0b;
        ret.ss = 0x1b;
        ret.es = 0x13;
        ret.ds = 0x13;
        ret.fs = 0x13;
        ret.gs = 0x13;

        ret
    }
}

pub static _TSS: TSS = TSS::new();