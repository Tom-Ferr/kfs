use crate::io::{inb, outb, insl};
use core::arch::asm;
use crate::printk;
use crate::timer::sleep;

extern "C" {
    fn pio_read(selector: u16, bus: u16, word: u16, esi: u32);
    fn pio_write(selector: u16, bus: u16, word: u16, esi: u32);
}

const ATA_SR_BSY  : u8  = 0x80;    // Busy
const ATA_SR_DRDY : u8  = 0x40;    // Drive ready
const ATA_SR_DF   : u8  = 0x20;    // Drive write fault
const ATA_SR_DSC  : u8  = 0x10;    // Drive seek complete
const ATA_SR_DRQ  : u8  = 0x08;    // Data request ready
const ATA_SR_CORR : u8  = 0x04;    // Corrected data
const ATA_SR_IDX  : u8  = 0x02;    // Index
const ATA_SR_ERR  : u8  = 0x01;    // Error

const ATA_ER_BBK   : u8 =  0x80;    // Bad block
const ATA_ER_UNC   : u8 =  0x40;    // Uncorrectable data
const ATA_ER_MC    : u8 =  0x20;    // Media changed
const ATA_ER_IDNF  : u8 =  0x10;    // ID mark not found
const ATA_ER_MCR   : u8 =  0x08;    // Media change request
const ATA_ER_ABRT  : u8 =  0x04;    // Command aborted
const ATA_ER_TK0NF : u8 =  0x02;    // Track 0 not found
const ATA_ER_AMNF  : u8 =  0x01;    // No address mark

const ATA_CMD_READ_PIO        : u8 = 0x20;
const ATA_CMD_READ_PIO_EXT    : u8 = 0x24;
const ATA_CMD_READ_DMA        : u8 = 0xC8;
const ATA_CMD_READ_DMA_EXT    : u8 = 0x25;
const ATA_CMD_WRITE_PIO       : u8 = 0x30;
const ATA_CMD_WRITE_PIO_EXT   : u8 = 0x34;
const ATA_CMD_WRITE_DMA       : u8 = 0xCA;
const ATA_CMD_WRITE_DMA_EXT   : u8 = 0x35;
const ATA_CMD_CACHE_FLUSH     : u8 = 0xE7;
const ATA_CMD_CACHE_FLUSH_EXT : u8 = 0xEA;
const ATA_CMD_PACKET          : u8 = 0xA0;
const ATA_CMD_IDENTIFY_PACKET : u8 = 0xA1;
const ATA_CMD_IDENTIFY        : u8 = 0xEC;

const ATAPI_CMD_READ  : u8 = 0xA8;
const ATAPI_CMD_EJECT : u8 = 0x1B;

const ATA_IDENT_DEVICETYPE   : u8 =  0;
const ATA_IDENT_CYLINDERS    : u8 =  2;
const ATA_IDENT_HEADS        : u8 =  6;
const ATA_IDENT_SECTORS      : u8 =  12;
const ATA_IDENT_SERIAL       : u8 =  20;
const ATA_IDENT_MODEL        : u8 =  54;
const ATA_IDENT_CAPABILITIES : u8 =  98;
const ATA_IDENT_FIELDVALID   : u8 =  106;
const ATA_IDENT_MAX_LBA      : u8 =  120;
const ATA_IDENT_COMMANDSETS  : u8 =  164;
const ATA_IDENT_MAX_LBA_EXT  : u8 =  200;

const IDE_ATA   : u8 = 0x00;
const IDE_ATAPI : u8 = 0x01;

const ATA_MASTER: u8 = 0x00;
const ATA_SLAVE : u8 = 0x01;

const ATA_REG_DATA       : u8 = 0x00;
const ATA_REG_ERROR      : u8 = 0x01;
const ATA_REG_FEATURES   : u8 = 0x01;
const ATA_REG_SECCOUNT0  : u8 = 0x02;
const ATA_REG_LBA0       : u8 = 0x03;
const ATA_REG_LBA1       : u8 = 0x04;
const ATA_REG_LBA2       : u8 = 0x05;
const ATA_REG_HDDEVSEL   : u8 = 0x06;
const ATA_REG_COMMAND    : u8 = 0x07;
const ATA_REG_STATUS     : u8 = 0x07;
const ATA_REG_SECCOUNT1  : u8 = 0x08;
const ATA_REG_LBA3       : u8 = 0x09;
const ATA_REG_LBA4       : u8 = 0x0A;
const ATA_REG_LBA5       : u8 = 0x0B;
const ATA_REG_CONTROL    : u8 = 0x0C;
const ATA_REG_ALTSTATUS  : u8 = 0x0C;
const ATA_REG_DEVADDRESS : u8 = 0x0D;

// Channels:
const ATA_PRIMARY   : u8 = 0x00;
const ATA_SECONDARY : u8 = 0x01;

// Directions:
const ATA_READ      : u8 = 0x00;
const ATA_WRITE     : u8 = 0x01;

static mut channels: [IDEChannelRegisters; 2] = [IDEChannelRegisters{base: 0, ctrl: 0, bmide: 0, nIEN: 0}; 2];
static mut  ide_devices: [IDEDevice; 4] = [IDEDevice {reserved : 0, channel : 0, drive : 0, dtype : 0, signature : 0, capabilities : 0, command_sets : 0, size : 0, model : [0;41]}; 4];

static mut ide_buf: [u8;2048] = [0;2048];
static mut ide_irq_invoked: u8 = 0;
static mut atapi_packet: [u8;12] = [0xA8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static mut package: [u8;2] = [0,0];

#[derive(Copy, Clone)]
struct IDEChannelRegisters {
    base : u16, // I/O Base.
    ctrl : u16, // Control Base
    bmide : u16, // Bus Master IDE
    nIEN : u8,  // nIEN (No Interrupt);
}

#[derive(Copy, Clone)]
struct IDEDevice {
    reserved : u8,      // 0 (Empty) or 1 (This Drive really exists).
    channel : u8,       // 0 (Primary Channel) or 1 (Secondary Channel).
    drive : u8,         // 0 (Master Drive) or 1 (Slave Drive).
    dtype : u16,         // 0: ATA, 1:ATAPI.
    signature : u16,    // Drive Signature
    capabilities : u16, // Features.
    command_sets : u32,  // Command Sets Supported.
    size : u32,         // Size in Sectors.
    model : [u8;41],     // Model in string.
}

pub unsafe fn ide_read(channel: usize, reg: u8) -> u8 {
    let mut result: u8 = 0;
    if reg > 0x07 && reg < 0x0C{
        ide_write(channel, ATA_REG_CONTROL, 0x80 | channels[channel].nIEN);
    }
    if reg < 0x08{
        result = inb(channels[channel].base + reg as u16 - 0x00);
    }
    else if reg < 0x0C{
        result = inb(channels[channel].base  + reg as u16 - 0x06);
    }
    else if reg < 0x0E{
        result = inb(channels[channel].ctrl  + reg as u16 - 0x0A);
    }
    else if reg < 0x16{
        result = inb(channels[channel].bmide + reg as u16 - 0x0E);
    }
    if reg > 0x07 && reg < 0x0C{
        ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);
    }
    result
}

pub unsafe fn ide_write(channel: usize, reg: u8, data: u8) {
    if reg > 0x07 && reg < 0x0C{
        ide_write(channel, ATA_REG_CONTROL, 0x80 | channels[channel].nIEN);
    }
    if reg < 0x08 {
        outb(channels[channel].base  + reg as u16 - 0x00, data);
    }
    else if reg < 0x0C {
        outb(channels[channel].base  + reg as u16 - 0x06, data);
    }
    else if reg < 0x0E {
        outb(channels[channel].ctrl  + reg as u16 - 0x0A, data);
    }
    else if reg < 0x16 {
        outb(channels[channel].bmide + reg as u16 - 0x0E, data);
    }
    if reg > 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);
    }
}

pub unsafe fn ide_read_buffer(channel: usize, reg: u16, buffer: &mut [u8], quads: u32) {
    /* WARNING: This code contains a serious bug. The inline assembly trashes ES and
    *           ESP for all of the code the compiler generates between the inline
    *           assembly blocks.
    */
    if reg > 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, 0x80 | channels[channel].nIEN);
    }

    asm!("
        push es
        mov ax, ds
        mov es, ax
    ");
    
    if reg < 0x08 {
        insl(channels[channel].base  + reg - 0x00, buffer, quads);
    }
    else if reg < 0x0C {
        insl(channels[channel].base  + reg - 0x06, buffer, quads);
    }
    else if reg < 0x0E {
        insl(channels[channel].ctrl  + reg - 0x0A, buffer, quads);
    }
    else if reg < 0x16 {
        insl(channels[channel].bmide + reg - 0x0E, buffer, quads);
    }
    asm!("pop es");
    if reg > 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);
    }
}

pub unsafe fn ide_polling(channel: usize, advanced_check: u32) -> Result<u8, u8> {
    // (I) Delay 400 nanosecond for BSY to be set:
    // -------------------------------------------------
    for _ in 0..4 {
        ide_read(channel , ATA_REG_ALTSTATUS); // Reading the Alternate Status port wastes 100ns; loop four times.
    }
 
    // (II) Wait for BSY to be cleared:
    // -------------------------------------------------
    while ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY > 0 { }
 
    if advanced_check > 0 {
        let state = ide_read(channel, ATA_REG_STATUS); // Read Status Register.
 
       // (III) Check For Errors:
       // -------------------------------------------------
        if state & ATA_SR_ERR > 0 {
           return Err(2); // Error.
        }
 
       // (IV) Check If Device fault:
       // -------------------------------------------------
        if state & ATA_SR_DF > 0 {
            return Err(1); // Device Fault.
        }
 
       // (V) Check DRQ:
       // -------------------------------------------------
       // BSY = 0; DF = 0; ERR = 0 so we should check for DRQ now.
        if (state & ATA_SR_DRQ) == 0 {
            return Err(3); // DRQ should be set
        }
 
    }
    Ok(0)
}

pub unsafe fn ide_print_error(drive: usize, mut err: u8) -> u8 {
    if err == 0 {
        return err;
    }
 
    printk!("IDE:");
    if err == 1 {
        printk!("- Device Fault\n     "); 
        err = 19;
    }
    else if err == 2 {
       let st = ide_read(ide_devices[drive].channel as usize, ATA_REG_ERROR);
       if st & ATA_ER_AMNF > 0 {printk!("- No Address Mark Found\n     ");   err = 7;}
       if st & ATA_ER_TK0NF > 0 {printk!("- No Media or Media Error\n     ");   err = 3;}
       if st & ATA_ER_ABRT > 0 {printk!("- Command Aborted\n     ");      err = 20;}
       if st & ATA_ER_MCR  > 0 {printk!("- No Media or Media Error\n     ");   err = 3;}
       if st & ATA_ER_IDNF > 0 {printk!("- ID mark not Found\n     ");      err = 21;}
       if st & ATA_ER_MC   > 0 {printk!("- No Media or Media Error\n     ");   err = 3;}
       if st & ATA_ER_UNC  > 0 {printk!("- Uncorrectable Data Error\n     ");   err = 22;}
       if st & ATA_ER_BBK  > 0 {printk!("- Bad Sectors\n     ");       err = 13;}
    } 
    else  if err == 3 {
        printk!("- Reads Nothing\n     "); err = 23;
    }
    else  if err == 4 {
        printk!("- Write Protected\n     "); err = 8;
    }

    let channel_labels = ["Primary", "Secondary"];
    let drive_labels = ["Master", "Slave"];
    let model_str = core::str::from_utf8(&ide_devices[drive].model).expect("Invalid UTF-8 data in array");
    printk!(ERROR, "- [{} {}] {}\n", channel_labels[ide_devices[drive].channel as usize], drive_labels[ide_devices[drive].drive as usize], model_str);
 
    err
}

pub unsafe fn ide_initialize(BAR0: u32, BAR1: u32, BAR2: u32, BAR3: u32, BAR4: u32) {
    
    let mut count: usize = 0;
    
    // 1- Detect I/O Ports which interface IDE Controller:
    channels[ATA_PRIMARY as usize].base  = ((BAR0 & 0xFFFFFFFC) + 0x1F0 * (!BAR0)) as u16;
    channels[ATA_PRIMARY as usize].ctrl  = ((BAR1 & 0xFFFFFFFC) + 0x3F6 * (!BAR1)) as u16;
    channels[ATA_SECONDARY as usize].base  = ((BAR2 & 0xFFFFFFFC) + 0x170 * (!BAR2)) as u16;
    channels[ATA_SECONDARY as usize].ctrl  = ((BAR3 & 0xFFFFFFFC) + 0x376 * (!BAR3)) as u16;
    channels[ATA_PRIMARY as usize].bmide = ((BAR4 & 0xFFFFFFFC) + 0) as u16; // Bus Master IDE
    channels[ATA_SECONDARY as usize].bmide = ((BAR4 & 0xFFFFFFFC) + 8) as u16; // Bus Master IDE

    // 2- Disable IRQs:
    ide_write(ATA_PRIMARY as usize, ATA_REG_CONTROL, 2);
    ide_write(ATA_SECONDARY as usize, ATA_REG_CONTROL, 2);

    // 3- Detect ATA-ATAPI Devices:
    for i in 0..2 {

        for j in 0..2 {
            
            let mut err = 0;
            let mut ide_type = IDE_ATA;
            ide_devices[count].reserved = 0; // Assuming that no drive here.
            
            // (I) Select Drive:
            ide_write(i as usize, ATA_REG_HDDEVSEL, 0xA0 | (j << 4)); // Select Drive.
            sleep(1); // Wait 1ms for drive select to work.
            
            // (II) Send ATA Identify Command:
            ide_write(i as usize, ATA_REG_COMMAND, ATA_CMD_IDENTIFY);
            sleep(1); // This function should be implemented in your OS. which waits for 1 ms.
            // it is based on System Timer Device Driver.
            
            // (III) Polling:
            if ide_read(i as usize, ATA_REG_STATUS) == 0 {
                continue;
            } // If Status = 0, No Device.
            
            loop{
                let status = ide_read(i as usize, ATA_REG_STATUS);
                if status & ATA_SR_ERR > 0 {
                    err = 1;
                    break;
                } // If Err, Device is not ATA.
                if !(status & ATA_SR_BSY) > 0 && (status & ATA_SR_DRQ) > 0 {
                    break; // Everything is right.
                } 
            }
            
            // (IV) Probe for ATAPI Devices:
            
            if err != 0 {
                let cl = ide_read(i as usize, ATA_REG_LBA1);
                let ch = ide_read(i as usize, ATA_REG_LBA2);
                
                if cl == 0x14 && ch == 0xEB {
                    ide_type = IDE_ATAPI;
                }
                else if cl == 0x69 && ch == 0x96 {
                    ide_type = IDE_ATAPI;
                }
                else{
                    continue; // Unknown Type (may not be a device).
                }
                
                ide_write(i as usize, ATA_REG_COMMAND, ATA_CMD_IDENTIFY_PACKET);
                sleep(1);
            }
            
            // (V) Read Identification Space of the Device:
            ide_read_buffer(i as usize, ATA_REG_DATA as u16, ide_buf.as_mut(), 128);
            
            // (VI) Read Device Parameters:
            ide_devices[count].reserved      = 1;
            ide_devices[count].dtype         = ide_type as u16;
            ide_devices[count].channel       = i;
            ide_devices[count].drive         = j;
            ide_devices[count].signature     = *((ide_buf.as_mut_ptr() as *mut u16).wrapping_add(ATA_IDENT_DEVICETYPE as usize));
            ide_devices[count].capabilities  = *((ide_buf.as_mut_ptr() as *mut u16).wrapping_add(ATA_IDENT_CAPABILITIES as usize));
            ide_devices[count].command_sets  = *((ide_buf.as_mut_ptr() as *mut u32).wrapping_add(ATA_IDENT_COMMANDSETS as usize));
            
            // (VII) Get Size:
            if ide_devices[count].command_sets & (1 << 26) > 0 {
                // Device uses 48-Bit Addressing:
                ide_devices[count].size   = *((ide_buf.as_mut_ptr() as *mut u32).wrapping_add(ATA_IDENT_MAX_LBA_EXT as usize));
            }
            else {
                // Device uses CHS or 28-bit Addressing:
                ide_devices[count].size   = *((ide_buf.as_mut_ptr() as *mut u32).wrapping_add(ATA_IDENT_MAX_LBA as usize));
            }
         
            // (VIII) String indicates model of device (like Western Digital HDD and SONY DVD-RW...):
            for k in (0..40).step_by(2) {
                ide_devices[count].model[k as usize] = ide_buf[(ATA_IDENT_MODEL + k + 1) as usize];
                ide_devices[count].model[k as usize + 1] = ide_buf[(ATA_IDENT_MODEL + k) as usize];
                ide_devices[count].model[40] = 0; // Terminate String.
             
                count += 1;
            }
        }    
    }

    // 4- Print Summary:
    let type_label = ["ATA", "ATAPI"];
    for i in 0..4{

        if (ide_devices[i as usize].reserved == 1) {
            let model_str = core::str::from_utf8(&ide_devices[i as usize].model).expect("Invalid UTF-8 data in array");
            printk!(" Found {} Drive {}GB - {}\n", type_label[ide_devices[i as usize].dtype as usize], ide_devices[i as usize].size / 1024 / 1024 / 2, model_str);
        }
    }
}

pub unsafe fn ide_ata_access(direction: u8, drive: usize, lba: u32, numsects: u8, selector: u16, mut edi: u32) -> Result<u8,u8> {
    let lba_mode: u8; /* 0: CHS, 1:LBA28, 2: LBA48 */;
    let dma: u8; /* 0: No DMA, 1: DMA */
    let mut cmd: u8 = 0;
    let mut lba_io: [u8;6] = [0;6];
    let channel = ide_devices[drive].channel as usize; // Read the Channel.
    let slavebit = ide_devices[drive].drive; // Read the Drive [Master/Slave]
    let bus: u16 = channels[channel].base; // Bus Base, like 0x1F0 which is also data port.
    let words: u16 = 256; // Almost every ATA drive has a sector-size of 512-byte.
    let cyl: u16;
    let head: u8;
    let sect: u8;

    ide_irq_invoked = 0x0;
    channels[channel].nIEN = ide_irq_invoked + 0x02;

    ide_write(channel, ATA_REG_CONTROL, channels[channel].nIEN);

     // (I) Select one from LBA28, LBA48 or CHS;
    if lba >= 0x10000000 { // Sure Drive should support LBA in this case, or you are
        // giving a wrong LBA.
        // LBA48:
        lba_mode  = 2;
        lba_io[0] = ((lba & 0x000000FF) >> 0) as u8;
        lba_io[1] = ((lba & 0x0000FF00) >> 8) as u8;
        lba_io[2] = ((lba & 0x00FF0000) >> 16) as u8;
        lba_io[3] = ((lba & 0xFF000000) >> 24) as u8;
        lba_io[4] = 0; // LBA28 is integer, so 32-bits are enough to access 2TB.
        lba_io[5] = 0; // LBA28 is integer, so 32-bits are enough to access 2TB.
        head      = 0; // Lower 4-bits of HDDEVSEL are not used here.
    }
    else if ide_devices[drive].capabilities & 0x200 > 0 { // Drive supports LBA?
        // LBA28:
        lba_mode  = 1;
        lba_io[0] = ((lba & 0x00000FF) >> 0) as u8;
        lba_io[1] = ((lba & 0x000FF00) >> 8) as u8;
        lba_io[2] = ((lba & 0x0FF0000) >> 16) as u8;
        lba_io[3] = 0; // These Registers are not used here.
        lba_io[4] = 0; // These Registers are not used here.
        lba_io[5] = 0; // These Registers are not used here.
        head      = ((lba & 0xF000000) >> 24) as u8;
    }
    else {
        // CHS:
        lba_mode  = 0;
        sect      = ((lba % 63) + 1) as u8;
        cyl       = ((lba + 1  - sect as u32) / (16 * 63)) as u16;
        lba_io[0] = sect;
        lba_io[1] = ((cyl >> 0) & 0xFF) as u8;
        lba_io[2] = ((cyl >> 8) & 0xFF) as u8;
        lba_io[3] = 0;
        lba_io[4] = 0;
        lba_io[5] = 0;
        head      = ((lba + 1  - sect as u32) % (16 * 63) / (63)) as u8; // Head number is written to HDDEVSEL lower 4-bits.
    }
    // (II) See if drive supports DMA or not;
    dma = 0; // We don't support DMA
    // (III) Wait if the drive is busy;
    while ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY > 0 {} // Wait if busy.
    // (IV) Select Drive from the controller;
    if lba_mode == 0 {
        ide_write(channel, ATA_REG_HDDEVSEL, 0xA0 | (slavebit << 4) | head); // Drive & CHS.
    }
    else{
        ide_write(channel, ATA_REG_HDDEVSEL, 0xE0 | (slavebit << 4) | head); // Drive & LBA
    }
    // (V) Write Parameters;
    if lba_mode == 2 {
        ide_write(channel, ATA_REG_SECCOUNT1,   0);
        ide_write(channel, ATA_REG_LBA3,   lba_io[3]);
        ide_write(channel, ATA_REG_LBA4,   lba_io[4]);
        ide_write(channel, ATA_REG_LBA5,   lba_io[5]);
    }
    ide_write(channel, ATA_REG_SECCOUNT0,   numsects);
    ide_write(channel, ATA_REG_LBA0,   lba_io[0]);
    ide_write(channel, ATA_REG_LBA1,   lba_io[1]);
    ide_write(channel, ATA_REG_LBA2,   lba_io[2]);

    // (VI) Select the command and send it;
    // Routine that is followed:
    // If ( DMA & LBA48)   DO_DMA_EXT;
    // If ( DMA & LBA28)   DO_DMA_LBA;
    // If ( DMA & LBA28)   DO_DMA_CHS;
    // If (!DMA & LBA48)   DO_PIO_EXT;
    // If (!DMA & LBA28)   DO_PIO_LBA;
    // If (!DMA & !LBA#)   DO_PIO_CHS;
    if lba_mode == 0 && dma == 0 && direction == 0 {
        cmd = ATA_CMD_READ_PIO;
    }
    if lba_mode == 1 && dma == 0 && direction == 0 {
        cmd = ATA_CMD_READ_PIO;   
    }
    if lba_mode == 2 && dma == 0 && direction == 0 {
        cmd = ATA_CMD_READ_PIO_EXT;   
    }
    if lba_mode == 0 && dma == 1 && direction == 0 {
        cmd = ATA_CMD_READ_DMA;
    }
    if lba_mode == 1 && dma == 1 && direction == 0 {
        cmd = ATA_CMD_READ_DMA;
    }
    if lba_mode == 2 && dma == 1 && direction == 0 {
        cmd = ATA_CMD_READ_DMA_EXT;
    }
    if lba_mode == 0 && dma == 0 && direction == 1 {
        cmd = ATA_CMD_WRITE_PIO;
    }
    if lba_mode == 1 && dma == 0 && direction == 1 {
        cmd = ATA_CMD_WRITE_PIO;
    }
    if lba_mode == 2 && dma == 0 && direction == 1 {
        cmd = ATA_CMD_WRITE_PIO_EXT;
    }
    if lba_mode == 0 && dma == 1 && direction == 1 {
        cmd = ATA_CMD_WRITE_DMA;
    }
    if lba_mode == 1 && dma == 1 && direction == 1 {
        cmd = ATA_CMD_WRITE_DMA;
    }
    if lba_mode == 2 && dma == 1 && direction == 1 {
        cmd = ATA_CMD_WRITE_DMA_EXT;
    }
    ide_write(channel, ATA_REG_COMMAND, cmd);               // Send the Command.

    if dma > 0 {

        if direction == 0 {}
        // DMA Read.
        else {}
        // DMA Write.
    }
    else {
        if direction == 0 {
           // PIO Read.
            for i in 0..numsects {
                if let Err(err) = ide_polling(channel, 1) {
                    return Err(err); // Polling, set error and exit if there is.
                }
                pio_read(selector, bus, words, edi);
                edi += (words*2) as u32;
            } 
        }
        else {
            // PIO Write.
            for i in 0..numsects {
                let _ = ide_polling(channel, 0); // Polling.
                pio_write(selector, bus, words, edi);
                edi += (words*2) as u32;
            }
            let cache = [ATA_CMD_CACHE_FLUSH, ATA_CMD_CACHE_FLUSH, ATA_CMD_CACHE_FLUSH_EXT];
            ide_write(channel, ATA_REG_COMMAND, cache[lba_mode as usize]);
            let _ = ide_polling(channel, 0); // Polling.
        }
    }
   Ok(0)
}

pub unsafe fn ide_read_sectors(drive: usize, numsects: u8, lba: u32, es: u16, edi: u32) {

    // 1: Check if the drive presents:
    // ==================================
    if drive > 3 || ide_devices[drive].reserved == 0 {
        package[0] = 0x1;      // Drive Not Found!
    }

    // 2: Check if inputs are valid:
    // ==================================
    else if (lba + numsects as u32) > ide_devices[drive].size && (ide_devices[drive].dtype == IDE_ATA as u16) {
        package[0] = 0x2;                     // Seeking to invalid position.
    }

    // 3: Read in PIO Mode through Polling & IRQs:
    // ============================================
    else {
        let mut err: u8 = 0;
        if (ide_devices[drive].dtype == IDE_ATA as u16) {
            err = ide_ata_access(ATA_READ, drive, lba, numsects, es, edi).unwrap();
        }
        // else if ide_devices[drive].Type == IDE_ATAPI {
        //     for (i = 0; i < numsects; i++)
        //     err = ide_atapi_read(drive, lba + i, 1, es, edi + (i*2048));
        // }
        package[0] = ide_print_error(drive, err);
    }
}

pub unsafe fn ide_write_sectors(drive: usize, numsects: u8, lba: u32, es: u16, edi: u32) {

    // 1: Check if the drive presents:
    // ==================================
    if drive > 3 || ide_devices[drive].reserved == 0 {
        package[0] = 0x1;      // Drive Not Found!
    }
    // 2: Check if inputs are valid:
    // ==================================
    else if ((lba + numsects as u32) > ide_devices[drive].size) && (ide_devices[drive].dtype == IDE_ATA as u16) {
        package[0] = 0x2;                     // Seeking to invalid position.
    }
    // 3: Read in PIO Mode through Polling & IRQs:
    // ============================================
    else {
        let mut err: u8 = 0;
        if ide_devices[drive].dtype == IDE_ATA as u16 {
            err = ide_ata_access(ATA_WRITE, drive, lba, numsects, es, edi).unwrap();
        }
        else if ide_devices[drive].dtype == IDE_ATAPI as u16 {
            err = 4; // Write-Protected.
        }
        package[0] = ide_print_error(drive, err);
    }
}