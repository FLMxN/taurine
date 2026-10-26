use crate::{read_tsc, u64_to_bytes};
use core3::io::{
    Cursor, Read as CoreRead, Seek as CoreSeek, SeekFrom as CoreSeekFrom, Write as CoreWrite,
};
use fatfs::{
    FileSystem, FormatVolumeOptions, FsOptions, IoBase, Read, ReadWriteSeek, Seek, SeekFrom, Write,
};
// use crate::Mutex;

pub const BLOCK_SECTOR_SIZE: usize = 512;

#[derive(Debug, Eq, PartialEq)]
pub enum BlockError {
    NoDevice,
    Timeout,
    DeviceFault,
    DeviceError(u8),
    LbaUnsupported,
    InvalidCapacity,
    InvalidBufferLength,
    TooManySectors,
    OutOfBounds,
}

pub trait BlockDevice {
    fn sector_count(&self) -> u32;
    fn read_sectors(&mut self, lba: u32, buffer: &mut [u8]) -> Result<(), BlockError>;
    fn write_sectors(&mut self, lba: u32, buffer: &[u8]) -> Result<(), BlockError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Superblock {
    pub inode_count: u32,
    pub block_count: u32,
    pub first_data_block: u32,
    pub block_size: u32,
    pub blocks_per_group: u32,
    pub inodes_per_group: u32,
    pub revision_level: u32,
    pub inode_size: u16,
    pub compatible_features: u32,
    pub incompatible_features: u32,
    pub read_only_features: u32,
}

#[derive(Debug)]
pub enum Ext2Error {
    Device(BlockError),
    InvalidMagic(u16),
    InvalidBlockSize(u32),
    InvalidInodeSize(u16),
    InvalidGeometry,
}

pub fn read_superblock<IO: BlockDevice>(device: &mut IO) -> Result<Superblock, Ext2Error> {
    const SUPERBLOCK_SIZE: usize = 1024;

    let mut raw = [0; SUPERBLOCK_SIZE];
    device
        .read_sectors(2, &mut raw)
        .map_err(Ext2Error::Device)?;

    let magic = u16::from_le_bytes([raw[56], raw[57]]);
    if magic != 0xef53 {
        return Err(Ext2Error::InvalidMagic(magic));
    }

    let log_block_size = u32_at(&raw, 24);
    if log_block_size > 6 {
        return Err(Ext2Error::InvalidBlockSize(log_block_size));
    }
    let block_size = 1024u32 << log_block_size;

    let revision_level = u32_at(&raw, 76);
    let inode_size = if revision_level == 0 {
        128
    } else {
        u16::from_le_bytes([raw[88], raw[89]])
    };
    if inode_size < 128 || u32::from(inode_size) > block_size || !inode_size.is_power_of_two() {
        return Err(Ext2Error::InvalidInodeSize(inode_size));
    }

    let superblock = Superblock {
        inode_count: u32_at(&raw, 0),
        block_count: u32_at(&raw, 4),
        first_data_block: u32_at(&raw, 20),
        block_size,
        blocks_per_group: u32_at(&raw, 32),
        inodes_per_group: u32_at(&raw, 40),
        revision_level,
        inode_size,
        compatible_features: u32_at(&raw, 92),
        incompatible_features: u32_at(&raw, 96),
        read_only_features: u32_at(&raw, 100),
    };

    if superblock.inode_count == 0
        || superblock.block_count <= superblock.first_data_block
        || superblock.blocks_per_group == 0
        || superblock.inodes_per_group == 0
    {
        return Err(Ext2Error::InvalidGeometry);
    }

    Ok(superblock)
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

pub struct FATBuffer<'a>(pub Cursor<&'a mut [u8]>);

impl<'a> IoBase for FATBuffer<'a> {
    type Error = ();
}

impl<'a> Read for FATBuffer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf).map_err(|_| ())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_exact(buf).map_err(|_| ())
    }
}

impl<'a> Write for FATBuffer<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf).map_err(|_| ())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buf).map_err(|_| ())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().map_err(|_| ())
    }
}

impl<'a> Seek for FATBuffer<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let core_pos = match pos {
            SeekFrom::Start(n) => CoreSeekFrom::Start(n),
            SeekFrom::End(n) => CoreSeekFrom::End(n),
            SeekFrom::Current(n) => CoreSeekFrom::Current(n),
        };
        self.0.seek(core_pos).map_err(|_| ())
    }
}

pub fn recall<IO>(
    fs: &FileSystem<IO>,
    path: &str,
    output: &mut [u8],
) -> Result<usize, fatfs::Error<IO::Error>>
where
    IO: ReadWriteSeek,
{
    let root = fs.root_dir();
    let mut file = root.open_file(path)?;
    let mut read = 0;

    while read < output.len() {
        let count = file.read(&mut output[read..])?;
        if count == 0 {
            break;
        }
        read += count;
    }

    Ok(read)
}

pub fn internalize<IO>(
    fs: &FileSystem<IO>,
    path: &str,
    contents: &[u8],
) -> Result<(), fatfs::Error<IO::Error>>
where
    IO: ReadWriteSeek,
{
    let root = fs.root_dir();
    let mut file = root.create_file(path)?;

    file.truncate()?;
    file.write_all(contents)?;
    file.flush()?;
    Ok(())
}
