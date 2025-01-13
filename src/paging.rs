#[derive(Copy, Clone, PartialEq)]
pub enum UserSpace{
    Kernel = 0x300,
    User = 0x0,
}

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct AlignedPage {
    pub data: [u32; 1024],
}

impl AlignedPage {
    pub const fn new() -> Self {
        Self {
            data: [0; 1024],
        }
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    pub pages: AlignedPage,
    pub bitmap: [u32; 32],
    pub size: usize,
}

impl PageTable {
    fn new(dir: &PageDirectory) -> &mut PageTable {
        unsafe{
            let pt: &PageTable = &*((((*dir.directory).data[(dir.whoami as usize) + dir.size] as u32 & 0xfffff000) + 0xC0000000) as *const PageTable);
            let frame: *const PageTable = (*pt).pages.data[1022] as *mut PageTable;
            let filtered_frame: *mut PageTable = ((frame as u32 & 0xfffff000) + 0xC0000000) as *mut PageTable;
            filtered_frame.write_volatile(Self {pages: AlignedPage::new(), bitmap: [0; 32], size: 0,});
            &mut(*filtered_frame)
        }
    }
}

#[repr(C)]
pub struct PageDirectory {
    pub directory: *mut AlignedPage,
    pub whoami: UserSpace,
    pub size: usize,
}

impl PageDirectory {
    const fn new(user: UserSpace) -> Self {
        Self {
            directory: &mut AlignedPage::new() as *mut AlignedPage,
            whoami: user,
            size: 0,
        }
    }

    const fn init(ptr: *const u32) -> Self{
        let aligned_page = unsafe { ptr as *mut AlignedPage };
        Self {
            directory: aligned_page,
            whoami: UserSpace::Kernel,
            size: 0,
        }
    }


    pub fn new_page(&mut self) -> Result<(),()>{
        
        if self.size >= 1024{
            return Err(());
        }
        
        let new_page: &mut PageTable = PageTable::new(self);
        let mut addr: u32 = 0;

        addr += (self.size + 1 * 0x400000) as u32;

        for i in 0..1024 {
            new_page.pages.data[i] = addr | 0x3;
            addr += 4096;
        }
        new_page.bitmap[31] |= 0b11;
        new_page.size = 1024;
        unsafe{
            (*self.directory).data[(self.whoami as usize) + self.size + 1] = (new_page as *const PageTable) as u32 | 0x3;
        }
        self.size += 1;
        Ok(())
    }
}

pub static mut DIR: Option<PageDirectory> = None;

pub fn init_page_tables(){
    unsafe{
        let dir = (crate::get_reg!(cr3) as u32 + 0xC0000000) as *const u32;
        DIR = Some(PageDirectory::init(dir));
    }
}