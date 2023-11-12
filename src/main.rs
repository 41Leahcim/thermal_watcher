use std::{
    fmt::Write as _,
    io::{self, Write as _},
    thread,
    time::Duration,
};

use sysinfo::{ComponentExt, System, SystemExt};

const DELAY: Duration = Duration::from_secs(1);

fn temperature_updater() {
    // Create a system instance and a string for the temperatures
    let mut sys = System::new_all();
    let mut temperatures = String::new();

    loop {
        // Wait until the temperatures have to be updated
        thread::park();

        // Update the temperatures
        sys.refresh_all();

        // Read the temperatures
        sys.components()
            .iter()
            .fold(&mut temperatures, |mut output, component| {
                writeln!(
                    &mut output,
                    "{}: {}",
                    component.label(),
                    component.temperature()
                )
                .unwrap();
                output
            });

        // Send the temperatures
        io::stdout().lock().write_all(temperatures.as_bytes()).ok();

        // Clear the temperatures buffer
        temperatures.clear();
    }
}

fn main() {
    // Create a ClearScreen object to clear the CLI more efficiently
    let clearscreen = clearscreen::ClearScreen::default();

    // Spawn a thread to update the temperatures
    let logger = thread::spawn(temperature_updater);

    // Log the temperatures every second
    loop {
        // Request a temperature measurement
        logger.thread().unpark();

        // Clear the screen
        clearscreen.clear_to(&mut io::stdout().lock()).unwrap();

        // Wait some time to save resources
        std::thread::sleep(DELAY);
    }
}
