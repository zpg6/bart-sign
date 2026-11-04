//! The settings record in the `config` flash partition.
//!
//! The partition is found by label in the partition table, with the offset
//! from `partitions.csv` as a fallback. The record format lives in
//! [`crate::settings`].

use core::cell::RefCell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_bootloader_esp_idf::partitions::{self, DataPartitionSubType, PartitionType};
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use log::{info, warn};

use crate::settings::{RECORD_LEN, Settings};

/// Must match the `config` row of `partitions.csv`.
const LABEL: &str = "config";
const FALLBACK_OFFSET: u32 = 0x31_0000;
const FALLBACK_LEN: u32 = 0x1000;

struct Store {
    flash: FlashStorage<'static>,
    offset: u32,
    len: u32,
}

static STORE: Mutex<CriticalSectionRawMutex, RefCell<Option<Store>>> = Mutex::new(RefCell::new(None));

#[derive(Debug)]
pub enum StoreError {
    NotInitialized,
    Flash(esp_storage::FlashStorageError),
}

/// Word-aligned so the flash ROM routines can use it without a bounce buffer.
#[repr(C, align(4))]
struct Record([u8; RECORD_LEN]);

/// Locate the partition and keep the flash driver for later saves.
pub fn init(flash: FLASH<'static>) {
    let mut flash = FlashStorage::new(flash);
    let (offset, len) = find_partition(&mut flash).unwrap_or_else(|| {
        warn!("no '{LABEL}' partition in the table, using {FALLBACK_OFFSET:#x}");
        (FALLBACK_OFFSET, FALLBACK_LEN)
    });
    info!("config partition at {offset:#x}, {len:#x} bytes");
    STORE.lock(|s| s.replace(Some(Store { flash, offset, len })));
}

fn find_partition(flash: &mut FlashStorage<'static>) -> Option<(u32, u32)> {
    let mut buf = alloc::vec![0u8; partitions::PARTITION_TABLE_MAX_LEN];
    let table = partitions::read_partition_table(flash, &mut buf)
        .inspect_err(|e| warn!("partition table: {e:?}"))
        .ok()?;
    table
        .iter()
        .find(|p| {
            p.label_as_str() == LABEL
                && p.partition_type() == PartitionType::Data(DataPartitionSubType::Undefined)
        })
        .map(|p| (p.offset(), p.len()))
}

/// The saved settings, or `None` if the partition is blank or the record is
/// invalid (wrong magic or version, bad CRC).
pub fn load() -> Option<Settings> {
    let mut rec = Record([0; RECORD_LEN]);
    STORE.lock(|s| {
        let mut s = s.borrow_mut();
        let s = s.as_mut()?;
        let offset = s.offset;
        // The NOR-flash trait methods, not FlashStorage's inherent ones: those
        // bounce every call through a 4 KB stack buffer.
        ReadNorFlash::read(&mut s.flash, offset, &mut rec.0)
            .inspect_err(|e| warn!("config read failed: {e:?}"))
            .ok()
    })?;
    Settings::decode(&rec.0)
}

/// Erase the partition and write `settings`, then read it back to check.
/// Blocks for ~50 ms with interrupts off (one 4 KB sector erase).
pub fn save(settings: &Settings) -> Result<(), StoreError> {
    let rec = Record(settings.encode());
    STORE.lock(|s| {
        let mut s = s.borrow_mut();
        let s = s.as_mut().ok_or(StoreError::NotInitialized)?;
        let (offset, len) = (s.offset, s.len);
        NorFlash::erase(&mut s.flash, offset, offset + len).map_err(StoreError::Flash)?;
        NorFlash::write(&mut s.flash, offset, &rec.0).map_err(StoreError::Flash)
    })?;
    match load() {
        Some(ref back) if back == settings => Ok(()),
        _ => Err(StoreError::Flash(esp_storage::FlashStorageError::IoError)),
    }
}

/// Wipe the settings: the next boot goes into setup mode.
pub fn erase() -> Result<(), StoreError> {
    STORE.lock(|s| {
        let mut s = s.borrow_mut();
        let s = s.as_mut().ok_or(StoreError::NotInitialized)?;
        let (offset, len) = (s.offset, s.len);
        NorFlash::erase(&mut s.flash, offset, offset + len).map_err(StoreError::Flash)
    })
}
