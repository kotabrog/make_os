#![no_std]
#![no_main]
#![feature(offset_of)]

use core::time::Duration;
use wasabi::executor::{Executor, Task, TimeoutFuture};
use wasabi::hpet::global_timestamp;
use wasabi::init::{init_allocator, init_basic_runtime, init_display, init_hpet, init_paging};
use wasabi::print::{hexdump, set_global_vram_writer};
use wasabi::uefi::{init_vram, locate_loaded_image_protocol, EfiHandle, EfiSystemTable};
use wasabi::x86::init_exceptions;
use wasabi::{error, info, println, warn};

#[no_mangle]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    println!("Booting WasabiOS...");
    println!("image_handle: {:#018X}", image_handle);
    println!("efi_system_table: {:#p}", efi_system_table);

    let loaded_image_protocol = locate_loaded_image_protocol(image_handle, efi_system_table)
        .expect("Failed to locate Loaded Image Protocol");
    println!("image_base: {:#018X}", loaded_image_protocol.image_base);
    println!("image_size: {:#018X}", loaded_image_protocol.image_size);

    info!("info");
    warn!("warn");
    error!("error");
    hexdump(efi_system_table);

    let mut vram = init_vram(efi_system_table).expect("Failed to initialize VRAM");

    init_display(&mut vram);

    set_global_vram_writer(vram);

    let acpi = efi_system_table.acpi_table().expect("ACPI table not found");

    let memory_map = init_basic_runtime(image_handle, efi_system_table);

    info!("Hello, Non-UEFI World!");

    init_allocator(&memory_map);

    let (_gdt, _idt) = init_exceptions();
    init_paging(&memory_map);

    init_hpet(acpi);

    let t0 = global_timestamp();

    let task1 = Task::new(async move {
        for i in 100..=103 {
            info!("{i} hpet.main_counter = {:?}", global_timestamp() - t0);
            TimeoutFuture::new(Duration::from_secs(1)).await;
        }
        Ok(())
    });
    let task2 = Task::new(async move {
        for i in 200..=203 {
            info!("{i} hpet.main_counter = {:?}", global_timestamp() - t0);
            TimeoutFuture::new(Duration::from_secs(1)).await;
        }
        Ok(())
    });
    let mut executor = Executor::new();
    executor.enqueue(task1);
    executor.enqueue(task2);
    Executor::run(executor);
}

#[cfg(all(not(test), not(feature = "std")))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info:?}");
    wasabi::qemu::exit_qemu(wasabi::qemu::QemuExitCode::Fail)
}
