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
    task::{executor::Executor, frame_counter, keyboard, Task},
    FRAMECOUNTER, GLOBALS,
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
    init(boot_info);
    command_line_logo();

    let mut globals = GLOBALS.lock();
    println!("CowOS ver: {}", globals.get("CowOS").unwrap());
    globals.insert("Loaded", "True");
    drop(globals);
    println!("{}", GLOBALS.is_locked());
    #[cfg(test)]
    test_main();
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    let lock = GLOBALS.lock();
    let globals = (*lock).clone();
    
    drop(lock);
    if globals.get("FrameCountEnabled").unwrap() == &"True" {
        executor.spawn(Task::new(frame_counter::print_frames()));
    }
    
    //frame start
    loop {
        executor.run();
        executor.sleep_if_idle();
        if globals.get("FrameCountEnabled").unwrap() == &"True" {
            let mut frame_counter = FRAMECOUNTER.lock();
            *frame_counter += 1;
            drop(frame_counter);
        }
    }
}
fn init(boot_info: &'static BootInfo) {
    cow_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
}
