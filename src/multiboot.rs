use crate::printf;

const MULTIBOOT_TAG_ALIGN                 :u32 =   8; 
const MULTIBOOT_TAG_TYPE_END              :u32 =   0; 
const MULTIBOOT_TAG_TYPE_CMDLINE          :u32 =   1; 
const MULTIBOOT_TAG_TYPE_BOOT_LOADER_NAME :u32 =   2; 
const MULTIBOOT_TAG_TYPE_MODULE           :u32 =   3; 
const MULTIBOOT_TAG_TYPE_BASIC_MEMINFO    :u32 =   4; 
const MULTIBOOT_TAG_TYPE_BOOTDEV          :u32 =   5; 
const MULTIBOOT_TAG_TYPE_MMAP             :u32 =   6; 
const MULTIBOOT_TAG_TYPE_VBE              :u32 =   7; 
const MULTIBOOT_TAG_TYPE_FRAMEBUFFER      :u32 =   8; 
const MULTIBOOT_TAG_TYPE_ELF_SECTIONS     :u32 =   9; 
const MULTIBOOT_TAG_TYPE_APM              :u32 =   10;
const MULTIBOOT_TAG_TYPE_EFI32            :u32 =   11;
const MULTIBOOT_TAG_TYPE_EFI64            :u32 =   12;
const MULTIBOOT_TAG_TYPE_SMBIOS           :u32 =   13;
const MULTIBOOT_TAG_TYPE_ACPI_OLD         :u32 =   14;
const MULTIBOOT_TAG_TYPE_ACPI_NEW         :u32 =   15;
const MULTIBOOT_TAG_TYPE_NETWORK          :u32 =   16;
const MULTIBOOT_TAG_TYPE_EFI_MMAP         :u32 =   17;
const MULTIBOOT_TAG_TYPE_EFI_BS           :u32 =   18;
const MULTIBOOT_TAG_TYPE_EFI32_IH         :u32 =   19;
const MULTIBOOT_TAG_TYPE_EFI64_IH         :u32 =   20;
const MULTIBOOT_TAG_TYPE_LOAD_BASE_ADDR   :u32 =   21;


const MULTIBOOT_MEMORY_AVAILABLE        :u32 =      1;
const MULTIBOOT_MEMORY_RESERVED         :u32 =      2;
const MULTIBOOT_MEMORY_ACPI_RECLAIMABLE :u32 =      3;
const MULTIBOOT_MEMORY_NVS              :u32 =      4;
const MULTIBOOT_MEMORY_BADRAM           :u32 =      5;

#[repr(C)]
struct MultiBootMmapEntry
{
  addr_low: u32,
  addr_high: u32,
  len_low: u32,
  len_high: u32,
  mem_type: u32,
  zero: u32,
}

#[repr(C)]
struct MultiBootTag
{
  tag_type: u32,
  size: u32,
}

#[repr(C)]
struct MultiBootTagMmap
{
  tag: MultiBootTag,
  entry_size: u32,
  entry_version: u32,
  entries: MultiBootMmapEntry, 
}

#[repr(C)]
struct MultiBootTagBasicMemInfo
{
  tag: MultiBootTag,
  mem_lower: u32,
  mem_upper: u32,
}

#[repr(C)]
struct MultiBootTagElfSections
{
  tag: MultiBootTag,
  num: u32,
  entsize: u32,
  shndx: u32,
  sections: *const u8,
}

pub fn read_multiboot_info(addr: u32) {

  let mut tag = addr + 8;
  loop {
    let current_tag = unsafe { &*(tag as *const MultiBootTag) };
    if current_tag.tag_type == MULTIBOOT_TAG_TYPE_END {
      break;
    }

    match current_tag.tag_type {
      MULTIBOOT_TAG_TYPE_BASIC_MEMINFO => {
        let mem_info_tag = unsafe { &*(tag as *const MultiBootTagBasicMemInfo) };
        let mem_lower = mem_info_tag.mem_lower;
        let mem_upper = mem_info_tag.mem_upper;
        printf!("mem_lower = {:x}, mem_upper = {:x}\n", mem_lower, mem_upper);
      }
      MULTIBOOT_TAG_TYPE_MMAP => {
        unsafe {
          let mmap_tag = &*(tag as *const MultiBootTagMmap);
          let mut mmap = &mmap_tag.entries as *const MultiBootMmapEntry;
          let end = tag + current_tag.size;
          
          while (mmap as u32) < end{
            printf!("base_addr = 0x{:x}, length = 0x{:x}, type = 0x{:x}\n", (*mmap).addr_low, (*mmap).len_low, (*mmap).mem_type);
            mmap = (mmap as u32 + mmap_tag.entry_size) as *const MultiBootMmapEntry;
          }
        }
      }
      _ => {}
    }
    tag += ((current_tag.size + 7) & !7);
  }

  let final_tag = unsafe{ &*(tag as *const MultiBootTag) };
  tag += ((final_tag.size + 7) & !7);
  crate::printf!("multiboot start = 0x{:x}\n", addr);
  crate::printf!("multiboot end = 0x{:x}\n", tag);
  printf!("Total mbi size {}\n", tag - addr);
}
