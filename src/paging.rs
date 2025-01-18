use crate::io::*;

#[derive(Copy, Clone, PartialEq)]
enum UserSpace{
    Kernel = 0x300,
    User = 0x0,
}

#[repr(C, align(4096))]
pub struct AlignedPage {
    data: [u32; 1024],
}

impl AlignedPage {
    pub const fn new() -> Self {
        Self {
            data: [0; 1024],
        }
    }

    pub const fn get_data(&self, index: usize) -> u32 {
        let dt = self.data[index];
        (dt & 0xfffff000) + 0xC0000000
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    pages: AlignedPage,
    pub bitmap: [u32; 32],
}

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

    pub fn set_frame(&mut self, index: usize, value: u32){
        self.pages.data[index] = value;
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
}

impl PageDirectory {
    const fn new(user: UserSpace) -> Self {
        Self {
            directory: &mut AlignedPage::new() as *mut AlignedPage,
            whoami: user,
            allocs: 0,
        }
    }

    const fn init(ptr: *const u32) -> Self{
        let aligned_page = unsafe { ptr as *mut AlignedPage };
        unsafe {
            let pt: &mut PageTable = &mut*((*aligned_page).get_data(UserSpace::Kernel as usize) as *mut PageTable);
            pt.bitmap[31] |= 0b11 << 30;
        }
        Self {
            directory: aligned_page,
            whoami: UserSpace::Kernel,
            allocs: 0,
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

    pub fn get_whoami(&self) -> usize{
        self.whoami as usize
    }

    pub fn set_page(&mut self, index: usize, value: u32){
        unsafe{
            (*self.directory).data[self.whoami as usize + index] = value;
        }
    }

    pub fn new_page(&mut self) -> Result<(),()>{
        
        if self.allocs >= 1024{
            return Err(());
        }
        
        let new_page: &mut PageTable = PageTable::new(self);
        let mut addr: u32 = 0;

        addr += (self.allocs + 1 * 0x400000) as u32;

        for i in 0..1024 {
            new_page.set_frame(i, addr | 0x3);
            addr += 4096;
        }
        new_page.bitmap[31] |= 0b11 << 30; //1022 % 32 = 30
        unsafe{
            self.set_page(self.allocs + 1, (new_page as *const PageTable) as u32 | 0x3);
        }
        self.allocs += 1;
        Ok(())
    }
}

pub static mut DIR: Option<PageDirectory> = None;
// pub static mut bitmap: [[u32; 32]; 1024] = [[0; 32]; 1024];

const FRAME_SIZE: u32 = 0x1000;
const TOTAL_MEMORY: u32 = FRAME_SIZE * 1024;
const NUMBER_OF_FRAMES: u32 = TOTAL_MEMORY / FRAME_SIZE;

pub fn alloc_page() -> Option<u32> {
    unsafe{
        let dir = DIR.as_mut().unwrap();

        for offset in 0..=dir.get_allocs(){
            let tab = &mut *(dir.get_page(offset as usize) as *mut PageTable);

            for i in 0..1024 {
                let byteIndex: usize = i / 32;
                let bitIndex: usize = i % 32;
                
                if (tab.bitmap[byteIndex] & (1 << bitIndex)) == 0 {
                    tab.bitmap[byteIndex] |= 1 << bitIndex;
                    return Some(tab.get_frame(i));
                }
            }
        }
        if let Ok(..) = dir.new_page() {
            let tab = &*(dir.get_page(dir.get_allocs()) as *const PageTable);
            return Some(tab.get_frame(0));
        }
    }
    None
}
    
pub fn free_page(ptr: u32) {
    unsafe {

        let dir = DIR.as_mut().unwrap();
        let offset = ptr / 0x400000;
        let tab = &mut*(dir.get_page(offset as usize) as *mut PageTable);
        let frameIndex: usize = ((ptr / FRAME_SIZE) % 1024) as usize;
        let byteIndex: usize = frameIndex / 32;
        let bitIndex: usize = frameIndex % 32;
        
        tab.bitmap[byteIndex] &= !(1 << bitIndex);
    }
}

pub fn block_pages(addr: u32, len: u32){
    unsafe{
        let dir = DIR.as_mut().unwrap();
        let end = addr + len;

        let begin_offset = addr / 0x400000;
        let end_offset = end / 0x400000;

        let begin_frameIndex = ((addr / FRAME_SIZE) % 1024);
        let end_frameIndex = ((end / FRAME_SIZE) % 1024);

        let begin_byteIndex = begin_frameIndex / 32;
        let end_byteIndex = end_frameIndex / 32;

        for offset in begin_offset..=end_offset{
            let tab = &mut*(dir.get_page(offset as usize) as *mut PageTable);
            for byteIndex in 0..32{
                match offset{
                    begin_offset if byteIndex == begin_byteIndex => {
                        let bitIndex = begin_frameIndex % 32;
                        tab.bitmap[byteIndex as usize] |= (!0) << bitIndex;
                    },
   
                    end_offset if byteIndex == end_byteIndex => {
                        let bitIndex = end_frameIndex % 32;
                        tab.bitmap[byteIndex as usize] |= !((!0) << bitIndex);
                    },

                    begin_offset if byteIndex < begin_byteIndex => continue,
                    
                    end_offset if byteIndex > end_byteIndex => break,

                    _ => tab.bitmap[byteIndex as usize] |= !0,
                }
            }
            
        }
    }
}

pub fn init_page_tables(){
    unsafe{
        let dir = (crate::get_reg!(cr3) as u32 + 0xC0000000) as *const u32;
        DIR = Some(PageDirectory::init(dir));
    }
}