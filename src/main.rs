use std::{
    fmt::Write,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use sysinfo::{ComponentExt, System, SystemExt};

const DELAY: Duration = Duration::from_secs(1);

fn temperature_updater(rx: Receiver<()>, tx: Sender<String>) {
    // Create a system instance and a string for the temperatures
    let mut sys = System::new_all();
    let mut temperatures = String::new();

    // Only perform an action when a request has been received
    for _ in rx {
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
        tx.send(temperatures.clone()).unwrap();

        // Clear the temperatures buffer
        temperatures.clear();
    }
}

fn main() {
    // Create a ClearScreen object to clear the CLI more efficiently
    let clearscreen = clearscreen::ClearScreen::default();

    // Create 2 channels, 1 for sending requests and 1 for receiving temperatues
    let (req_tx, req_rx) = channel();
    let (result_tx, result_rx) = channel();

    // Spawn a thread to update the temperatures
    thread::spawn(|| temperature_updater(req_rx, result_tx));

    // Request a temperature measurement
    req_tx.send(()).unwrap();

    // Log the temperatures every second
    loop {
        // Start measuring performance
        let start = Instant::now();

        // Clear the screen
        clearscreen.clear().unwrap();

        // Receive the temperatures
        let temperatures = result_rx.recv().unwrap();

        // Print the readings and performance
        println!("{temperatures}{}", start.elapsed().as_secs_f64());

        // Request a temperature measurement
        req_tx.send(()).unwrap();

        // Wait some time to save resources
        std::thread::sleep(DELAY);
    }
}
