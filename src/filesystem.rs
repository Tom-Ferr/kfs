const EXT2_ERRORS_CONTINUE: u32 =	1;
const EXT2_ERRORS_RO: u32 =	2;
const EXT2_ERRORS_PANIC: u32 =	3;

const EXT2_OS_LINUX: u32 =	0;
const EXT2_OS_HURD: u32 =	1;
const EXT2_OS_MASIX: u32 =	2;
const EXT2_OS_FREEBSD: u32 = 3;
const EXT2_OS_LITES4: u32 =	4;

const EXT2_GOOD_OLD_REV: u32 = 0;
const EXT2_DYNAMIC_REV: u32 = 1;

const EXT2_BAD_INO: u32 = 0x01;
const EXT2_ROOT_INO: u32 = 0x02;
const EXT2_ACL_IDX_INO: u32 = 0x03;
const EXT2_ACL_DATA_INO: u32 = 0x04;
const EXT2_BOOT_LOADER_INO: u32 = 0x05;
const EXT2_UNDEL_DIR_INO: u32 = 0x06;

const EXT2_S_IFMT: u32 = 0xF000;
const EXT2_S_IFSOCK: u32 = 0xC000;
const EXT2_S_IFLNK: u32 = 0xA000;
const EXT2_S_IFREG: u32 = 0x8000;
const EXT2_S_IFBLK: u32 = 0x6000;
const EXT2_S_IFDIR: u32 = 0x4000;
const EXT2_S_IFCHR: u32 = 0x2000;
const EXT2_S_IFIFO: u32 = 0x1000;

const EXT2_S_ISUID: u32 = 0x0800;
const EXT2_S_ISGID: u32 = 0x0400;
const EXT2_S_ISGID: u32 = 0x0200;
const EXT2_S_IRWXU: u32 = 0x01C0;
const EXT2_S_IRUSR: u32 = 0x0100;
const EXT2_S_IWUSR: u32 = 0x0080;
const EXT2_S_IXUSR: u32 = 0x0040;
const EXT2_S_IRWXG: u32 = 0x0038;
const EXT2_S_IRGRP: u32 = 0x0020;
const EXT2_S_IWGRP: u32 = 0x0010;
const EXT2_S_IXGRP: u32 = 0x0008;
const EXT2_S_IRWXO: u32 = 0x0007;
const EXT2_S_IROTH: u32 = 0x0004;
const EXT2_S_IWOTH: u32 = 0x0002;
const EXT2_S_IXOTH: u32 = 0x0001;

const EXT2_FT_UNKNOWN: u32 = 0;
const EXT2_FT_REG_FILE: u32 = 1;
const EXT2_FT_DIR: u32 = 2;
const EXT2_FT_CHRDEV: u32 = 3;
const EXT2_FT_BLKDEV: u32 = 4;
const EXT2_FT_FIFO: u32 = 5;
const EXT2_FT_SOCK: u32 = 6;
const EXT2_FT_SYMLINK: u32 = 7;
const EXT2_FT_MAX: u32 = 8;

#[repr(C, packed)]
pub struct SuperBlock {

    s_inodes_count: u32,
    s_blocks_count: u32,
    s_r_blocks_count: u32,
    s_free_blocks_count: u32,
    s_free_inodes_count: u32,
    s_first_data_block: u32,
    s_log_block_size: u32,
    s_log_frag_size: u32,
    s_blocks_per_group: u32,
    s_frags_per_group: u32,
    s_inodes_per_group: u32,
    s_mtime: u32,
    s_wtime: u32,
    s_mnt_count: u16,
    s_max_mnt_count: u16,
    s_magic: u16,
    s_state: u16,
    s_errors: u16,
    s_minor_rev_level: u16,
    s_lastcheck: u32,
    s_checkinterval: u32,
    s_creator_os: u32,
    s_rev_level: u32,
    s_def_resuid: u16,
    s_def_resgid: u16,

    s_first_ino: u32,
    s_inode_size: u16,
    s_block_group_nr: u16,
    s_feature_compat: u32,
    s_feature_incompat: u32,
    s_feature_ro_compat: u32,
    s_uuid: [u8; 16],
    s_volume_name: [u8; 16],
    s_last_mounted: [u8; 64],
    s_algo_bitmap: u32,

    s_prealloc_blocks: u8,
    s_prealloc_dir_blocks: u8,
    _padding206: [u8; 2],

    s_journal_uuid: [u8; 16],
    s_journal_inum: u32,
    s_journal_dev: u32,
    s_last_orphan: u32,

    _padding236: [u8; 788],
}

#[repr(C, packed)]
pub struct GroupDescriptor{
    bg_block_bitmap: u32,
    bg_inode_bitmap: u32,
    bg_inode_table: u32,
    bg_free_blocks_count: u16,
    bg_free_inodes_count: u16,
    bg_used_dirs_count: u16,
    bg_pad: u16,
    bg_reserved: [u8;12],
}
    
#[repr(C, packed)]
pub struct Inode {
    i_mode: u16,
    i_uid: u16,
    i_size: u32,
    i_atime: u32,
    i_ctime: u32,
    i_mtime: u32,
    i_dtime: u32,
    i_gid: u16,
    i_links_count: u16,
    i_blocks: u32,
    i_flags: u32,
    i_osd1: u32,
    i_block: [u32; 15],
    i_generation: u32,
    i_file_acl: u32,
    i_dir_acl: u32,
    i_faddr: u32,
    i_osd2: Osd2,
}

#[repr(C, packed)]
pub struct Osd2 {
    h_i_frag: u8,
    h_i_fsize: u8,
    h_i_mode_high: u16,
    h_i_uid_high: u16,
    h_i_gid_high: u16,
    h_i_author: u32,
}

#[repr(C, packed)]
pub struct DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    name: [u8; 255],
}
