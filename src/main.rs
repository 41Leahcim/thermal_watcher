use std::{
    fmt::Write,
    time::{Duration, Instant},
};

use sysinfo::{ComponentExt, System, SystemExt};

const DELAY: Duration = Duration::from_secs(1);

fn main() {
    let mut sys = System::new_all();

    // Create a ClearScreen object to clear the CLI more efficiently
    let clearscreen = clearscreen::ClearScreen::default();

    // Log the temperatures every second
    loop {
        // Start measuring performance
        let start = Instant::now();

        // Clear the screen
        clearscreen.clear().unwrap();

        sys.refresh_all();
        let temperatures = sys
            .components()
            .iter()
            .fold(String::new(), |mut output, component| {
                writeln!(
                    &mut output,
                    "{}: {}",
                    component.label(),
                    component.temperature()
                )
                .unwrap();
                output
            });

        // Print the readings and performance
        println!("{temperatures}{}", start.elapsed().as_secs_f64());

        std::thread::sleep(DELAY);
    }
}
