#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(cow_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#[allow(dead_code)] //For use by vga to control output. Could be used to define serial or vga
const DISPLAY_MODE: u8 = 0;
mod serial;
mod vga_buffer; //Handles display
use core::panic::PanicInfo;
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
    )
}
//Entrance
#[no_mangle]
pub extern "C" fn _start() -> ! {
    command_line_logo();
    cow_os::init();
    #[cfg(test)]
    test_main();
    println!("Reached end of kernel. Halting...");
    cow_os::hlt_loop()
}
