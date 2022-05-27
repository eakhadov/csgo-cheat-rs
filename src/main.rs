#![cfg(target_os = "windows")]

#[macro_use]
extern crate log;

pub mod mem;

use std::process::exit;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SPACE};
use windows::{core::*, Win32::System::Console::FreeConsole};

fn main() -> Result<()> {
    // unsafe {
    //     FreeConsole();
    // }

    env_logger::init();
    info!("starting up");

    let process = mem::from_name("csgo.exe").ok_or_else(|| exit(1)).unwrap();

    let client = process.get_module("client.dll").unwrap();
    // let engine = process.get_module("engine.dll").unwrap();

    // let local_player = process.read::<usize>(client.base + 0xDBA5BC).unwrap();
    // println!("local_player = {}", local_player);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1));

        for id in 1..66 {
            let enemy = process.read::<usize>(client.base + 0x4DD69DC + (0x10 * (id - 1))).unwrap();
            process.write::<usize>(enemy + 0x93D, &1);
        }

        // let local_player = match process.read::<usize>(client.base + 0xDBA5BC) {
        //     Some(p) => p,
        //     _ => continue,
        // };

        // let flags = process.read::<usize>(local_player + 0x104).unwrap();

        // if unsafe { GetAsyncKeyState(32) != 0 } && (flags & 1) != 0 {
        //     process.write::<usize>(client.base + 0x5280924, &6);
        // }
    }

    Ok(())
}
