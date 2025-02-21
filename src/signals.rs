use crate::queue::*;

pub type SignalQueue = Queue<Signal>;

#[repr(i32)]
pub enum Sig {
    Hangup         = 1, 
    Interrupt      = 2, 
    Quit           = 3, 
    Illegal        = 4, 
    Trap           = 5, 
    Abort          = 6, 
    Bus            = 7, 
    FloatingPoint  = 8, 
    Kill           = 9, 
    User1          = 10,
    Segmentation   = 11,
    User2          = 12,
    Pipe           = 13,
    Alarm          = 14,
    Terminate      = 15,
    StkFlt         = 16,
    Child          = 17,
    Continue       = 18,
    Stop           = 19,
    Suspend        = 20,
    TerminalIn     = 21,
    TerminalOut    = 22,
    Urgent         = 23,
    CpuTime        = 24,
    FileSize       = 25,
    VirtualAlarm   = 26,
    Profiling      = 27,
    WindowChange   = 28,
    Ipc            = 29,
    Power          = 30,
    Sys            = 31,
}

pub struct Signal{
    signal: Sig,
    next: *const Signal
}

impl Queuable for Signal {
    type Ptr = Signal;
    fn get_next(&self) -> *const Signal{
        self.next
    }
    fn set_next(&mut self, value: *const Signal){
        self.next = value;
    }
}