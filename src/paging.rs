use crate::io::*;

pub static mut DIR: Option<PageDirectory> = None;

pub const FRAME_SIZE: u32 = 0x1000;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum UserSpace{
    Virtual = 0x8c,
    Kernel = 0x300,
    User = 0x0,
}

#[repr(C, align(4096))]
struct AlignedPage {
    data: [u32; 1024],
}

impl AlignedPage {
    const fn new() -> Self {
        Self {
            data: [0; 1024],
        }
    }

    const fn get_data(&self, index: usize) -> u32 {
        let dt = self.data[index];
        (dt & 0xfffff000) + 0xC0000000
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    pages: AlignedPage,
    pub bitmap: [u32; 32],
}

#[allow(dead_code)]
impl PageTable {
    fn new(dir: &PageDirectory) -> &mut PageTable {
        unsafe{
            let pt: &PageTable = &*(dir.get_page(dir.get_allocs()) as *const PageTable);
            let frame: *mut PageTable = pt.get_frame(1022) as *mut PageTable;
            frame.write_volatile(Self {pages: AlignedPage::new(), bitmap: [0; 32],});
            &mut(*frame)
        }
    }

    pub fn get_frame(&self, index: usize) -> u32{
        let frame = self.pages.data[index];
        (frame & 0xfffff000) + 0xC0000000
    }

    pub fn set_frame(&mut self, index: usize, value: u32, flags: u32){
        self.pages.data[index] = value | flags;
    }

    pub fn print_bitmap(&self){
        clear_vga();
        enable_cursor(false);
        for i in 0..32{
            crate::printf!("index: {:02}, {:032b}\n", i, self.bitmap[i as usize]);

            if get_cursor() == 24 * 160 {
                crate::printf!("Press \'ENTER\' to continue, \'ESC\' to quit");
                loop{
                    let scan_code = read_key();
                    if scan_code == 0x01{
                        clear_vga();
                        set_cursor(0);
                        enable_cursor(true);
                        return ;
                    }
                    else if scan_code == 0x1C{
                        clear_vga();
                        set_cursor(0);
                        break ;
                    }
                }
            }
        }

        crate::printf!("Press \'ENTER\' to quit");
        loop{
            let scan_code = read_key();
            if scan_code == 0x1C{
                clear_vga();
                set_cursor(0);
                enable_cursor(true);
                break ;
            }
        }
    }
}

#[repr(C)]
pub struct PageDirectory {
    directory: *mut AlignedPage,
    whoami: UserSpace,
    allocs: usize,
    virtual_allocs: usize,
}

#[allow(dead_code)]
impl PageDirectory {
    pub fn init(&mut self, addr: u32) {
        let usr = addr as *mut AlignedPage;
        
        unsafe{
            #[allow(static_mut_refs)]
            let kern = (*DIR.as_mut().unwrap()).directory;
            (*usr).data = (*kern).data;
        }
        
        self.directory = usr;
        self.whoami = UserSpace::User;
        self.allocs = 0;
        self.virtual_allocs = 0;
    }

    const fn new(ptr: *const u32) -> Self{
        let aligned_page = ptr as *mut AlignedPage;
        unsafe {
            let pt: &mut PageTable = &mut*((*aligned_page).get_data(UserSpace::Kernel as usize) as *mut PageTable);
            pt.bitmap[31] |= 0b11 << 30;
        }
        Self {
            directory: aligned_page,
            whoami: UserSpace::Kernel,
            allocs: 0,
            virtual_allocs: 0,
        }
    }

    pub fn get_page(&self, offset: usize) -> u32{
        unsafe{
            let pt = (*self.directory).data[self.whoami as usize + offset];
            (pt & 0xfffff000) + 0xC0000000
        }
    }

    pub fn get_allocs(&self) -> usize{
        self.allocs
    }

    pub fn get_virtual_allocs(&self) -> usize{
        self.virtual_allocs
    }

    pub fn get_whoami(&self) -> usize{
        self.whoami as usize
    }

    pub fn set_page(&mut self, index: usize, value: u32, flags: u32){
        unsafe{
            (*self.directory).data[self.whoami as usize + index] = (value - 0xC0000000) | flags;
        }
    }

    pub fn new_page(&mut self) -> Result<(),()>{
        
        if self.allocs >= 140{
            return Err(());
        }
        
        let new_page: &mut PageTable = PageTable::new(self);
        let mut addr: u32 = 0;

        addr += (self.allocs + 1 * 0x400000) as u32;

        for i in 0..1024 {
            new_page.set_frame(i, addr, 0x3);
            addr += 4096;
        }
        new_page.bitmap[31] |= 0b11 << 30; //1022 % 32 = 30
        self.set_page(self.allocs + 1, (new_page as *const PageTable) as u32, 0x3);
        self.allocs += 1;
        Ok(())
    }

    pub fn new_virtual_page(&mut self, offset: usize) -> Result<(),()>{
        unsafe{
            if let Some(page) = alloc_page(size_of::<PageTable>()){
                let pg = page as *mut PageTable;
                pg.write_volatile(PageTable {pages: AlignedPage::new(), bitmap: [0; 32],});
                self.set_page(UserSpace::Virtual as usize + offset, page, 0x3);
                self.virtual_allocs += 1;
            }
            else{
                return Err(());
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn alloc_page(nbytes: usize) -> Option<u32> {
    let mut npages = nbytes as u32 / FRAME_SIZE;
    if nbytes as u32 % FRAME_SIZE != 0{
        npages += 1;
    }
    let limit = 32 - npages;
    let cursor = !0 >> limit;
    unsafe{
        #[allow(static_mut_refs)]
        let dir = DIR.as_mut().unwrap();

        for offset in 0..=dir.get_allocs(){
            let tab = &mut *(dir.get_page(offset as usize) as *mut PageTable);

            for i in 0..1024 {
                let byte_index: usize = i / 32;
                let bit_index: usize = i % 32;

                if bit_index > limit as usize{
                    continue;
                }
                
                let target = cursor << bit_index;
                if (tab.bitmap[byte_index] & target) == 0 {
                    tab.bitmap[byte_index] |= target;
                    return Some(tab.get_frame(i));
                }
            }
        }
        if let Ok(..) = dir.new_page() {
            let tab = &mut *(dir.get_page(dir.get_allocs()) as *mut PageTable);
            tab.bitmap[dir.get_allocs()] |= cursor;
            return Some(tab.get_frame(0));
        }
    }
    None
}

#[allow(dead_code)]
pub fn free_page(ptr: u32) {
    unsafe {

        #[allow(static_mut_refs)]
        let dir = DIR.as_mut().unwrap();
        let offset = ptr / 0x400000;
        let tab = &mut*(dir.get_page(offset as usize) as *mut PageTable);
        let frame_index: usize = ((ptr / FRAME_SIZE) % 1024) as usize;
        let byte_index: usize = frame_index / 32;
        let bit_index: usize = frame_index % 32;
        
        tab.bitmap[byte_index] &= !(1 << bit_index);
    }
}

#[allow(dead_code)]
pub fn block_pages(addr: u32, len: u32){
    unsafe{
        #[allow(static_mut_refs)]
        let dir = DIR.as_mut().unwrap();
        let end = addr + len;

        let _begin_offset = addr / 0x400000;
        let _end_offset = end / 0x400000;

        let begin_frame_index = (addr / FRAME_SIZE) % 1024;
        let end_frame_index = (end / FRAME_SIZE) % 1024;

        let begin_byte_index = begin_frame_index / 32;
        let end_byte_index = end_frame_index / 32;

        for offset in _begin_offset..=_end_offset{
            let tab = &mut*(dir.get_page(offset as usize) as *mut PageTable);
            for byte_index in 0..32{
                match offset{
                    _begin_offset if byte_index == begin_byte_index => {
                        let bit_index = begin_frame_index % 32;
                        tab.bitmap[byte_index as usize] |= (!0) << bit_index;
                    },
   
                    _end_offset if byte_index == end_byte_index => {
                        let bit_index = end_frame_index % 32;
                        tab.bitmap[byte_index as usize] |= !((!0) << bit_index + 1);
                    },

                    _begin_offset if byte_index < begin_byte_index => continue,
                    
                    _end_offset if byte_index > end_byte_index => break,

                    _ => tab.bitmap[byte_index as usize] |= !0,
                }
            }
            
        }
    }
}

pub fn init_page_tables(){
    unsafe{
        let dir = (crate::get_reg!(cr3) as u32 + 0xC0000000) as *const u32;
        DIR = Some(PageDirectory::new(dir));
    }
}