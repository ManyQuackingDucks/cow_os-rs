#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::empty_loop)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
extern crate alloc;
use alloc::collections::BTreeMap;
#[allow(unused_imports)]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

lazy_static! {
    pub static ref GLOBALS: Mutex<BTreeMap<&'static str, &'static str>> = Mutex::new({
        let mut m = BTreeMap::new();
        m.insert("CowOS","0.1.0");
        m.insert("Loaded","False");
        m.insert("DisplayMode","0"); //For use by vga to control output. Could be used to define serial or vga
        m.insert("FrameCountEnabled","False");
        m.insert("PrintOnTimerInt","False");
        m
    });
    pub static ref FRAMECOUNTER: Mutex<u8> = Mutex::new(0);
}


#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

#[cfg(test)]
entry_point!(test_kernel_entr);

#[cfg(test)]
/// Entry point for `cargo test`
fn test_kernel_entr(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop()
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
