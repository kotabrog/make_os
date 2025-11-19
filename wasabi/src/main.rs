#![no_std]
#![no_main]
#![feature(offset_of)]

use core::fmt::Write;
use core::writeln;
use wasabi::graphics::{draw_test_pattern, fill_rect, Bitmap};
use wasabi::print::hexdump;
use wasabi::{println, info, warn, error};
use wasabi::init::init_basic_runtime;
use wasabi::uefi::{init_vram, EfiHandle, EfiMemoryType, EfiSystemTable, VramTextWriter};
use wasabi::x86::hlt;

#[no_mangle]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    println!("Booting WasabiOS...");
    println!("image_handle: {:#018X}", image_handle);
    println!("efi_system_table: {:#p}", efi_system_table);
    info!("info");
    warn!("warn");
    error!("error");
    hexdump(efi_system_table);

    let mut vram = init_vram(efi_system_table).expect("Failed to initialize VRAM");

    let vw = vram.width();
    let vh = vram.height();
    fill_rect(&mut vram, 0x000000, 0, 0, vw, vh).expect("Failed to fill rect");

    draw_test_pattern(&mut vram);

    let mut w = VramTextWriter::new(&mut vram);

    let memory_map = init_basic_runtime(image_handle, efi_system_table);

    let mut total_memory_pages = 0;
    for e in memory_map.iter() {
        if e.memory_type() != EfiMemoryType::CONVENTIONAL_MEMORY {
            continue;
        }
        total_memory_pages += e.number_of_pages();
        writeln!(w, "{e:?}").unwrap();
    }
    let total_memory_size_mib = total_memory_pages * 4096 / 1024 / 1024;
    writeln!(
        w,
        "Total: {total_memory_pages} pages = {total_memory_size_mib} MiB"
    )
    .unwrap();

    writeln!(w, "Hello, Non-UEFI World!").unwrap();

    let cr3 = wasabi::x86::read_cr3();
    println!("CR3: {cr3:#p}");
    hexdump(unsafe { &*cr3 });

    loop {
        hlt()
    }
}

#[cfg(all(not(test), not(feature = "std")))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info:?}");
    wasabi::qemu::exit_qemu(wasabi::qemu::QemuExitCode::Fail)
}
