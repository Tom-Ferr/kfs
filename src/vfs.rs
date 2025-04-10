const FS_FILE       : u32 = 0x01;
const FS_DIRECTORY  : u32 = 0x02;
const FS_CHARDEVICE : u32 = 0x03;
const FS_BLOCKDEVICE: u32 = 0x04;
const FS_PIPE       : u32 = 0x05;
const FS_SYMLINK    : u32 = 0x06;
const FS_MOUNTPOINT : u32 = 0x08;

struct DIRENT {
    name: [u8; 255],
    ino: u32,
}

struct FsNode {
    name: [u8;255],
    mask: u32,
    uid: u32,
    gid: u32,
    flags: u32,
    inode: u32,
    length: u32,
    implementation: u32,
    read: Option< fn(*const FsNode, u32, u32, *const u8) -> Option<u32> >,
    write: Option< fn(*const FsNode, u32, u32, *const u8) -> Option<u32> >,
    open: Option<fn(*const FsNode)>,
    close: Option<fn(*const FsNode)>,
    readdir: Option< fn(*const FsNode, u32) -> Option<*const DIRENT> >, // Returns the n'th child of a directory.
    finddir: Option< fn(*const FsNode, *const u8) -> Option<*const FsNode> >,// Try to find a child in a directory by name.
    ptr: *const FsNode,
}

pub unsafe fn read_fs(node: *const FsNode, offset: u32, size: u32, buffer: *const u8) -> Option<u32> {
    if let Some(callback) = (*node).read {
        return Some(callback(node, offset, size, buffer).unwrap());
    }
    None
}

pub unsafe fn write_fs(node: *const FsNode, offset: u32, size: u32, buffer: *const u8) -> Option<u32> {
    if let Some(callback) = (*node).write {
        return Some(callback(node, offset, size, buffer).unwrap());
    }
    None
}

pub unsafe fn open_fs(node: *const FsNode) {
    if let Some(callback) = (*node).open {
        callback(node);
    }
}

pub unsafe fn close_fs(node: *const FsNode) {
    if let Some(callback) = (*node).close {
        callback(node);
    }
}

pub unsafe fn readdir_fs(node: *const FsNode, index: u32) -> Option<*const DIRENT> {
    if (*node).flags & 0x7 == FS_DIRECTORY {
        if let Some(callback) = (*node).readdir {
            return Some(callback(node, index).unwrap());
        }
    }
    None
}

pub unsafe fn finddir_fs(node: *const FsNode, name: *const u8) -> Option<*const FsNode> {
    if (*node).flags & 0x7 == FS_DIRECTORY {
        if let Some(callback) = (*node).finddir {
            return Some(callback(node, name).unwrap());
        }
    }
    None
}