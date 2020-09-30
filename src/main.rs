use std::process::{Command, exit};
use std::time::Duration;
use std::thread;

use futures::executor::block_on;
use pidfd::PidFd;

#[macro_use]
extern crate clap;

mod sleep_inhibitor;
mod dbus_sleep_inhibitor;

use crate::sleep_inhibitor::{SleepInhibitor, platform::*};

fn can_parse<T: std::str::FromStr>(val: String) -> Result<(), String> {
    match val.parse::<T>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value wasn't a valid {}", std::any::type_name::<T>()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap_app!(caffeinate =>
        (version: env!("CARGO_PKG_VERSION"))
        (about: "Prevent the system from sleeping on behalf of a utility")
        (@arg display: -d conflicts_with[idle] "Prevents the display from sleeping")
        (@arg idle: -i conflicts_with[display] "Prevents the system from idle sleeping")
        (@arg TIMEOUT: -t +takes_value {can_parse::<u64>} conflicts_with[PID] "Specifies timeout value in seconds to wait for while preventing sleep")
        (@arg PID: -w +takes_value {can_parse::<libc::pid_t>} conflicts_with[TIMEOUT] "Specifies PID of a process to wait for while preventing sleep")
        (@arg utility: ... conflicts_with[TIMEOUT PID] "Utility arguments")
    ).get_matches();

    let mut inhibitor: Box<dyn SleepInhibitor> = if matches.is_present("display") {
        Box::new(DisplaySleepInhibitor::new()
            .map_err(|e| {
                eprintln!("Inhibiting display sleep is unsupported");
                e
            }
        )?)
    } else {
        Box::new(SystemSleepInhibitor::new()
            .map_err(|e| {
                eprintln!("Inhibiting system idle sleep is unsupported");
                e
            }
        )?)
    };

    if let Ok(timeout) = value_t!(matches, "TIMEOUT", u64) {
        inhibitor.inhibit("caffeinate", format!("Inhibiting sleep for {} seconds", timeout).as_str());
        thread::sleep(Duration::from_secs(timeout));
        return Ok(());
    } else if let Ok(pid) = value_t!(matches, "PID", libc::pid_t) {
        inhibitor.inhibit("caffeinate", format!("Inhibiting sleep until process {} exits", pid).as_str());
        match unsafe { PidFd::open(pid, 0) } {
            Ok(pidfd) => { block_on(pidfd.into_future()).unwrap(); },
            Err(e) => {
                eprintln!("{}: {}", pid, e);
                exit(1);
            }
        }
    } else if let Some(utility) = matches.values_of("utility") {
        let args: Vec<&str> = utility.collect();
        inhibitor.inhibit(args[0], "Inhibiting sleep until process exits");
        let status = Command::new(args[0])
                             .args(&args[1..])
                             .status();
        match status {
            Ok(status) => exit(status.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("{}: {}", args[0], e);
                exit(127);
            }
        }
    } else {
        inhibitor.inhibit("caffeinate", "Inhibiting sleep forever");
        loop { thread::park(); };
    }

    Ok(())
}
