extern crate alloc;

use crate::acpi::AcpiRsdpStruct;
use crate::allocator::ALLOCATOR;
use crate::graphics::{draw_test_pattern, fill_rect, Bitmap};
use crate::hpet::{set_global_hpet, Hpet};
use crate::info;
use crate::uefi::{
    exit_from_efi_boot_services, EfiHandle, EfiMemoryType, EfiMemoryType::*, EfiSystemTable,
    MemoryMapHolder, VramBufferInfo,
};
use crate::x86::{write_cr3, PageAttr, PAGE_SIZE, PML4};
use alloc::boxed::Box;
use core::cmp::max;

pub fn init_basic_runtime(
    image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
) -> MemoryMapHolder {
    let mut memory_map = MemoryMapHolder::new();
    exit_from_efi_boot_services(image_handle, efi_system_table, &mut memory_map);
    ALLOCATOR.init_with_mmap(&memory_map);
    memory_map
}

pub fn init_paging(memory_type: &MemoryMapHolder) {
    let mut table = PML4::new();
    let mut end_of_mem = 0x1_0000_0000u64; // 4 GiB
    for e in memory_type.iter() {
        match e.memory_type() {
            CONVENTIONAL_MEMORY | LOADER_CODE | LOADER_DATA => {
                end_of_mem = max(
                    end_of_mem,
                    e.physical_start() + e.number_of_pages() * (PAGE_SIZE as u64),
                );
            }
            _ => (),
        }
    }
    table
        .create_mapping(0, end_of_mem, 0, PageAttr::ReadWriteKernel)
        .expect("Failed to create initial page mapping");
    // Unmap page 0 to detect null ptr dereference
    table
        .create_mapping(0, 4096, 0, PageAttr::NotPresent)
        .expect("Failed to unmap page 0");
    unsafe {
        write_cr3(Box::into_raw(table));
    }
}

pub fn init_hpet(acpi: &AcpiRsdpStruct) {
    let hpet = acpi.hpet().expect("Failed to get HPET from ACPI");
    let hpet = hpet
        .base_address()
        .expect("Failed to get HPET base address");
    info!("HPET is at {hpet:#p}");
    let hpet = Hpet::new(hpet);
    set_global_hpet(hpet);
}

pub fn init_allocator(memory_map: &MemoryMapHolder) {
    let mut total_memory_pages = 0;
    for e in memory_map.iter() {
        if e.memory_type() != EfiMemoryType::CONVENTIONAL_MEMORY {
            continue;
        }
        total_memory_pages += e.number_of_pages();
        info!("{e:?}");
    }
    let total_memory_size_mib = total_memory_pages * 4096 / 1024 / 1024;
    info!("Total memory: {total_memory_pages} pages = {total_memory_size_mib} MiB");
}

pub fn init_display(vram: &mut VramBufferInfo) {
    let vw = vram.width();
    let vh = vram.height();
    fill_rect(vram, 0x000000, 0, 0, vw, vh).expect("Failed to fill rect");
    draw_test_pattern(vram);
}
