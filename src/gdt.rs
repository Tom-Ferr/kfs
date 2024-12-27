#[repr(packed)]
struct Gdt{
    limit: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    flags: u8,
    base_high: u8,
}

impl Gdt {
    pub fn new(base: u32, limit: u32, access: u8, gran: u8) -> Self {
        Gdt{
            base_low: base & 0xFFFF,
            base_middle: (base >> 16) & 0xFF,
            base_high: (base >> 24) & 0xFF,
            limit: limit & 0xFFFF,
            flags: (limit >> 16) & 0x0F,
            access: access,
        }
        Gdt.flags |= gran & 0xF0;
    }
}
// null_desciptor = Gdt::new(0, 0x0, 0x0, 0x0);
// kernel_code = Gdt::new(0, 0xFFFFFFFF, 0x9A, 0xCF);
// kernel_data = Gdt::new(0, 0xFFFFFFFF, 0x92, 0xCF);
// user_code = Gdt::new(0, 0xFFFFFFFF, 0xFA, 0xCF);
// user_data = Gdt::new(0, 0xFFFFFFFF, 0xF2, 0xCF);
