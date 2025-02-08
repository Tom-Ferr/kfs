use core::mem::size_of;
use crate::tss::*;

const GDT_ADDR: u32 = 0xC0000800;

extern "C" {
    fn gdt_flush(gdt_ptr: *const Gdtr);
}

#[repr(C, packed)]
struct GdtDescriptor{
    limit: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    flags: u8,
    base_high: u8,
}

impl GdtDescriptor {
    fn new(base: u32, limit: u32, access: u8, granularity : u8) -> Self {
        Self{
            limit: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access: access,
            flags: (((limit >> 16) & 0x0F) as u8 | (granularity  & 0xF0)),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[repr(C, packed)]
struct GdtEntries{
    null_desciptor: GdtDescriptor,
    kernel_code: GdtDescriptor,
    kernel_data: GdtDescriptor,
    kernel_stack: GdtDescriptor,
    user_code: GdtDescriptor,
    user_data: GdtDescriptor,
    user_stack: GdtDescriptor,
    tss: GdtDescriptor,
}

impl GdtEntries {
    fn new() -> Self {
        unsafe{

            let tss_addr: u32 = _TSS.as_ref() as *const TSS as u32;
            let tss_size: u32 = size_of::<TSS>() as u32 - 1;
            Self{
                null_desciptor: GdtDescriptor::new(0x0, 0x0, 0x0, 0x0),
                kernel_code: GdtDescriptor::new(0x0, 0xFFFFFFFF, 0x9A, 0xCF),
                kernel_data: GdtDescriptor::new(0x0, 0xFFFFFFFF, 0x92, 0xCF),
                kernel_stack: GdtDescriptor::new(0x0, 0xFFFFFFFF, 0x96, 0xCF),
                user_code: GdtDescriptor::new(0x0, 0xFFFFFFFF, 0xFA, 0xCF),
                user_data: GdtDescriptor::new(0x0, 0xFFFFFFFF, 0xF2, 0xCF),
                user_stack: GdtDescriptor::new(0x0, 0xFFFFFFFF, 0xF6, 0xCF),
                tss: GdtDescriptor::new(tss_addr, tss_addr+tss_size, 0x89, 0xCF),
            }
        }
    }
}

#[repr(C, packed)]
struct Gdtr{
    limit: u16,
    base: *const GdtEntries,
}

impl Gdtr {
    fn new(entries: *mut GdtEntries) -> Self {
        Self{
            limit: (size_of::<GdtEntries>() - 1) as u16,
            base: entries as *const GdtEntries,
        }
    }
}

pub fn init_gdt() {
    
    unsafe{
        _TSS.init();
        let gdt_entries = GdtEntries::new();
        let gdt_addr = GDT_ADDR as *mut GdtEntries;
        gdt_addr.write_volatile(gdt_entries); // safer then *gdt_addr = gdt_entries;
        let gdtr = Gdtr::new(gdt_addr);

        gdt_flush(&gdtr);
    }
}