use core::arch::asm;

use crate::electrons::dopamine::{BLOCK_SECTOR_SIZE, BlockDevice, BlockError};

const DATA: u16 = 0x1f0;
const ERROR_FEATURES: u16 = 0x1f1;
const SECTOR_COUNT: u16 = 0x1f2;
const LBA_LOW: u16 = 0x1f3;
const LBA_MID: u16 = 0x1f4;
const LBA_HIGH: u16 = 0x1f5;
const DRIVE_HEAD: u16 = 0x1f6;
const STATUS_COMMAND: u16 = 0x1f7;
const ALT_STATUS_CONTROL: u16 = 0x3f6;

const MAX_LBA28_SECTORS: u32 = 1 << 28;
const MAX_COMMAND_SECTORS: u32 = 256;
const STATUS_ERROR: u8 = 1 << 0;
const STATUS_DEVICE_FAULT: u8 = 1 << 5;
const STATUS_DATA_REQUEST: u8 = 1 << 3;
const STATUS_BUSY: u8 = 1 << 7;
const POLL_LIMIT: usize = 1_000_000;

pub struct PrimaryMaster {
    sector_count: u32,
    supports_flush: bool,
}

impl PrimaryMaster {
    pub fn identify() -> Result<Self, BlockError> {
        unsafe {
            write_u8(ALT_STATUS_CONTROL, 0x02);
            write_u8(DRIVE_HEAD, 0xa0);
            io_wait();
        }
        wait_until_idle()?;
        unsafe {
            write_u8(SECTOR_COUNT, 0);
            write_u8(LBA_LOW, 0);
            write_u8(LBA_MID, 0);
            write_u8(LBA_HIGH, 0);
            write_u8(STATUS_COMMAND, 0xec);
            io_wait();
        }

        wait_for_data()?;
        let mut identify = [0u16; 256];
        for word in &mut identify {
            *word = unsafe { read_u16(DATA) };
        }

        if identify[49] & (1 << 9) == 0 {
            return Err(BlockError::LbaUnsupported);
        }

        let sector_count = u32::from(identify[60]) | (u32::from(identify[61]) << 16);
        if sector_count == 0 {
            return Err(BlockError::InvalidCapacity);
        }

        Ok(Self {
            sector_count: sector_count.min(MAX_LBA28_SECTORS),
            supports_flush: identify[82] & (1 << 12) != 0,
        })
    }

    fn validate_request(&self, lba: u32, buffer_len: usize) -> Result<u32, BlockError> {
        if buffer_len % BLOCK_SECTOR_SIZE != 0 {
            return Err(BlockError::InvalidBufferLength);
        }

        let sectors = u32::try_from(buffer_len / BLOCK_SECTOR_SIZE)
            .map_err(|_| BlockError::TooManySectors)?;
        if sectors > MAX_COMMAND_SECTORS {
            return Err(BlockError::TooManySectors);
        }
        if lba
            .checked_add(sectors)
            .is_none_or(|end| end > self.sector_count)
        {
            return Err(BlockError::OutOfBounds);
        }

        Ok(sectors)
    }

    fn select_lba(&self, lba: u32, sectors: u32) -> Result<(), BlockError> {
        unsafe {
            write_u8(DRIVE_HEAD, 0xe0 | ((lba >> 24) as u8 & 0x0f));
            io_wait();
        }
        wait_until_idle()?;
        unsafe {
            write_u8(SECTOR_COUNT, sectors as u8);
            write_u8(LBA_LOW, lba as u8);
            write_u8(LBA_MID, (lba >> 8) as u8);
            write_u8(LBA_HIGH, (lba >> 16) as u8);
        }
        Ok(())
    }
}

impl BlockDevice for PrimaryMaster {
    fn sector_count(&self) -> u32 {
        self.sector_count
    }

    fn read_sectors(&mut self, lba: u32, buffer: &mut [u8]) -> Result<(), BlockError> {
        let sectors = self.validate_request(lba, buffer.len())?;
        if sectors == 0 {
            return Ok(());
        }

        self.select_lba(lba, sectors)?;
        unsafe {
            write_u8(STATUS_COMMAND, 0x20);
            io_wait();
        }
        for sector in buffer.chunks_exact_mut(BLOCK_SECTOR_SIZE) {
            wait_for_data()?;
            for pair in sector.chunks_exact_mut(2) {
                let value = unsafe { read_u16(DATA) }.to_le_bytes();
                pair.copy_from_slice(&value);
            }
        }
        wait_until_idle()
    }

    fn write_sectors(&mut self, lba: u32, buffer: &[u8]) -> Result<(), BlockError> {
        let sectors = self.validate_request(lba, buffer.len())?;
        if sectors == 0 {
            return Ok(());
        }

        self.select_lba(lba, sectors)?;
        unsafe {
            write_u8(STATUS_COMMAND, 0x30);
            io_wait();
        }
        for sector in buffer.chunks_exact(BLOCK_SECTOR_SIZE) {
            wait_for_data()?;
            for pair in sector.chunks_exact(2) {
                unsafe {
                    write_u16(DATA, u16::from_le_bytes([pair[0], pair[1]]));
                }
            }
        }
        wait_until_idle()?;

        if self.supports_flush {
            unsafe {
                write_u8(STATUS_COMMAND, 0xe7);
                io_wait();
            }
            wait_until_idle()?;
        }
        Ok(())
    }
}

fn wait_for_data() -> Result<(), BlockError> {
    wait_status(STATUS_DATA_REQUEST)
}

fn wait_until_idle() -> Result<(), BlockError> {
    for _ in 0..POLL_LIMIT {
        let status = unsafe { read_u8(STATUS_COMMAND) };
        if status == 0 || status == 0xff {
            return Err(BlockError::NoDevice);
        }
        if status & STATUS_BUSY != 0 {
            continue;
        }
        if status & STATUS_ERROR != 0 {
            return Err(BlockError::DeviceError(unsafe { read_u8(ERROR_FEATURES) }));
        }
        if status & STATUS_DEVICE_FAULT != 0 {
            return Err(BlockError::DeviceFault);
        }
        if status & STATUS_DATA_REQUEST == 0 {
            return Ok(());
        }
    }
    Err(BlockError::Timeout)
}

fn wait_status(required: u8) -> Result<(), BlockError> {
    for _ in 0..POLL_LIMIT {
        let status = unsafe { read_u8(STATUS_COMMAND) };
        if status == 0 || status == 0xff {
            return Err(BlockError::NoDevice);
        }
        if status & STATUS_BUSY != 0 {
            continue;
        }
        if status & STATUS_ERROR != 0 {
            return Err(BlockError::DeviceError(unsafe { read_u8(ERROR_FEATURES) }));
        }
        if status & STATUS_DEVICE_FAULT != 0 {
            return Err(BlockError::DeviceFault);
        }
        if status & required == required {
            return Ok(());
        }
    }
    Err(BlockError::Timeout)
}

unsafe fn io_wait() {
    for _ in 0..4 {
        unsafe {
            read_u8(ALT_STATUS_CONTROL);
        }
    }
}

unsafe fn read_u8(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack));
    }
    value
}

unsafe fn write_u8(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    }
}

unsafe fn read_u16(port: u16) -> u16 {
    let value: u16;
    unsafe {
        asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack));
    }
    value
}

unsafe fn write_u16(port: u16, value: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
    }
}
