use crate::paging::*;

const FRAME_SIZE: u32 = 0x1000;
static TOTAL_MEMORY: u32 = 4096 * 1024;
static NUMBER_OF_FRAMES: u32 = TOTAL_MEMORY / FRAME_SIZE;

fn alloc_page() -> Option<u32> {
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
    
fn free_page(ptr: u32) {
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
                if byteIndex == (begin_byteIndex) && offset == begin_offset{
                    let bitIndex = begin_frameIndex % 32;
                    tab.bitmap[byteIndex as usize] |= (!0) << bitIndex;
                }
                else if byteIndex == (end_byteIndex) && offset == end_offset{
                    let bitIndex = end_frameIndex % 32;
                    tab.bitmap[byteIndex as usize] |= !((!0) << bitIndex);
                }
                else if offset == begin_offset && byteIndex < begin_byteIndex{
                    continue ;
                }
                else if offset == end_offset && byteIndex > end_byteIndex{
                    break ;
                }
                else{
                    tab.bitmap[byteIndex as usize] |= !0;
                }
            }
            
        }
    }
}
