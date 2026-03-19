use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use tinyboot::traits::boot::Storage as StorageTrait;

use tinyboot_ch32_hal::flash::FlashWriter;

const FLASH_WRITE_SIZE: usize = 2;
const FLASH_ERASE_SIZE: usize = 64;

pub struct StorageConfig {
    pub boot_base: u32,
    pub boot_size: u32,
    pub app_base: u32,
    pub app_size: usize,
}

#[derive(Debug)]
pub enum StorageError {
    NotAligned,
    OutOfBounds,
    Protected,
}

impl NorFlashError for StorageError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            StorageError::NotAligned => NorFlashErrorKind::NotAligned,
            StorageError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            StorageError::Protected => NorFlashErrorKind::Other,
        }
    }
}

pub struct Storage {
    boot_base: u32,
    boot_size: u32,
    app_base: u32,
    app_size: usize,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Self {
        Storage {
            boot_base: config.boot_base,
            boot_size: config.boot_size,
            app_base: config.app_base,
            app_size: config.app_size,
        }
    }

    fn app_ptr(&self) -> *const u8 {
        self.app_base as *const u8
    }
}

impl ErrorType for Storage {
    type Error = StorageError;
}

impl NorFlash for Storage {
    const WRITE_SIZE: usize = FLASH_WRITE_SIZE;
    const ERASE_SIZE: usize = FLASH_ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if !(from as usize).is_multiple_of(FLASH_ERASE_SIZE)
            || !(to as usize).is_multiple_of(FLASH_ERASE_SIZE)
        {
            return Err(StorageError::NotAligned);
        }
        if to as usize > self.app_size {
            return Err(StorageError::OutOfBounds);
        }
        let writer = FlashWriter::standard();
        writer.erase_start();
        let mut addr = self.app_base + from;
        let end = self.app_base + to;
        while addr < end {
            writer.erase(addr);
            addr += FLASH_ERASE_SIZE as u32;
        }
        writer.operation_end();
        #[cfg(debug_assertions)]
        if writer.check_wrprterr() {
            return Err(StorageError::Protected);
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if !(offset as usize).is_multiple_of(FLASH_WRITE_SIZE)
            || !bytes.len().is_multiple_of(FLASH_WRITE_SIZE)
        {
            return Err(StorageError::NotAligned);
        }
        if offset as usize + bytes.len() > self.app_size {
            return Err(StorageError::OutOfBounds);
        }
        let writer = FlashWriter::standard();
        writer.write_start();
        let mut addr = self.app_base + offset;
        for pair in bytes.chunks_exact(2) {
            let halfword = u16::from_le_bytes([pair[0], pair[1]]);
            writer.write(addr, halfword);
            addr += 2;
        }
        writer.operation_end();

        #[cfg(debug_assertions)]
        if writer.check_wrprterr() {
            return Err(StorageError::Protected);
        }
        Ok(())
    }
}

impl StorageTrait for Storage {
    fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.app_ptr(), self.app_size) }
    }

    fn boot_base(&self) -> usize {
        self.boot_base as usize
    }

    fn boot_size(&self) -> usize {
        self.boot_size as usize
    }

    fn unlock(&mut self) {
        tinyboot_ch32_hal::flash::unlock();
    }
}

impl ReadNorFlash for Storage {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.app_size {
            return Err(StorageError::OutOfBounds);
        }
        let src = unsafe { core::slice::from_raw_parts(self.app_ptr(), self.app_size) };
        let offset = offset as usize;
        bytes.copy_from_slice(&src[offset..offset + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.app_size
    }
}
