#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(cow_os::test_runner)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![reexport_test_harness_main = "test_main"]
mod serial;
mod vga_buffer; //Handles display
extern crate alloc;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use cow_os::{
    allocator,
    memory::{self, BootInfoFrameAllocator},
    task::{executor::Executor, keyboard, Task},
    vga_buffer::clear,
    GLOBALS,
};
use x86_64::VirtAddr;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    cow_os::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cow_os::test_panic_handler(info)
}
pub fn command_line_logo() {
    println!(
        r#"








    ______   ______   __     __       ______   ______    
   /\  ___\ /\  __ \ /\ \  _ \ \     /\  __ \ /\  ___\   
   \ \ \____\ \ \/\ \\ \ \/ ".\ \    \ \ \/\ \\ \___  \  
    \ \_____\\ \_____\\ \__/".~\_\    \ \_____\\/\_____\ 
     \/_____/ \/_____/ \/_/   \/_/     \/_____/ \_____/




   "#
    );
}


//Entrance
entry_point!(kernel_entr);
#[no_mangle]
fn kernel_entr(boot_info: &'static BootInfo) -> ! {
    command_line_logo();
    println!("Starting up");
    cow_os::init();
    clear();
    command_line_logo();
    println!("Initializing memory");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    clear();
    #[cfg(test)]
    test_main();
    let mut globals = GLOBALS.lock();
    globals.insert("Loaded", "True");
    command_line_logo();
    println!("Cow OS Version: {}", globals.get("CowOS").unwrap());
    drop(globals);
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    //frame start
    loop {
        executor.run();
        executor.sleep_if_idle();
    }
}