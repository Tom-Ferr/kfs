
extern "C" {
    fn enable_paging(dir_ptr: *const PageTable);
}

#[repr(C, align(4096))]
pub struct AlignedPage {
    pub data: [u32; 1024],
}

impl AlignedPage {
    const fn new() -> Self {
        Self {
            data: [0; 1024],
        }
    }
}

#[repr(C)]
pub struct PageTable {
    pub tables: AlignedPage,
    size: u32,
}

type PageDirectory = PageTable;

impl PageTable {
    const fn new() -> Self {
        Self {
            tables: AlignedPage::new(),
            size: 0,
        }
    }

    fn insert(&mut self, pt: &PageTable) {
        if self.size <= 1024 {
            self.tables.data[self.size as usize] = &pt.tables.data as *const u32 as u32 | 0x3;
            self.size += 1;
        }
    }

    fn fill(&mut self, start_addr: u32){
        let mut addr = start_addr as *const u32;
        for i in 0..1024 {
            self.tables.data[i] = addr as u32 | 0x3;
            addr = addr.wrapping_add(1024);
        }
        self.size = 1024;
    }
}

pub static mut DIR: PageDirectory = PageDirectory::new();
pub static mut TABLE: PageTable = PageTable::new();

pub fn init_page_tables(){
    unsafe{

        TABLE.fill(0x0);
        DIR.insert(&TABLE);
        enable_paging(&DIR);
    }
}