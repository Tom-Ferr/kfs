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

pub fn init_page_tables(){
    unsafe{
        let dir = (crate::get_reg!(cr3) as u32 + 0xC0000000) as *const u32;
        DIR = Some(PageDirectory::init(dir));
    }
}