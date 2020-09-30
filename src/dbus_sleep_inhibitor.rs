use std::error::Error;
use std::os::unix::io::RawFd;
use std::time::Duration;

use dbus::arg::OwnedFd;
use dbus::blocking::Connection;
use crate::sleep_inhibitor::SleepInhibitor;

pub struct DBusSystemSleepInhibitor {
    conn: Connection,
    fd: Option<RawFd>
}

impl DBusSystemSleepInhibitor {
    pub fn new() -> Result<DBusSystemSleepInhibitor, Box<dyn Error>> {
        let conn = Connection::new_system()?;
        let mut inhibitor = DBusSystemSleepInhibitor {
            conn: conn,
            fd: None
        };

        inhibitor.try_inhibit("caffeinate", "Testing inhibit")?;
        inhibitor.try_uninhibit()?;
        Ok(inhibitor)
    }

    fn try_inhibit(&mut self, application: &str, reason: &str) -> Result<(), dbus::Error> {
        let proxy = self.conn.with_proxy("org.freedesktop.login1", "/org/freedesktop/login1",
                                         Duration::from_millis(1000));
        let (fd, ): (OwnedFd, ) = proxy.method_call("org.freedesktop.login1.Manager", "Inhibit",
                                                    ("idle", application, reason, "block"))?;
        self.fd = Some(fd.into_fd());
        Ok(())
    }

    fn try_uninhibit(&mut self) -> Result<(), Box<dyn Error>> {
        let fd = self.fd.ok_or_else(|| "fd was None")?;
        nix::unistd::close(fd)?;
        self.fd = None;
        Ok(())
    }
}

impl SleepInhibitor for DBusSystemSleepInhibitor {
    fn inhibit(&mut self, application: &str, reason: &str) {
        self.try_inhibit(application, reason).unwrap()
    }

    fn uninhibit(&mut self) {
        self.try_uninhibit().unwrap()
    }
}

pub struct DBusDisplaySleepInhibitor {
    conn: Connection,
    cookie: Option<u32>
}

impl DBusDisplaySleepInhibitor {
    pub fn new() -> Result<DBusDisplaySleepInhibitor, Box<dyn Error>> {
        let conn = Connection::new_session()?;
        let mut inhibitor = DBusDisplaySleepInhibitor {
            conn: conn,
            cookie: None
        };

        inhibitor.try_inhibit("caffeinate", "Testing inhibit")?;
        inhibitor.try_uninhibit()?;
        Ok(inhibitor)
    }

    fn try_inhibit(&mut self, application: &str, reason: &str) -> Result<(), dbus::Error> {
        let proxy = self.conn.with_proxy("org.freedesktop.ScreenSaver", "/ScreenSaver",
                                         Duration::from_secs(1));
        let (cookie, ): (u32, ) = proxy.method_call("org.freedesktop.ScreenSaver", "Inhibit",
                                                    (application, reason))?;
        self.cookie = Some(cookie);
        Ok(())
    }

    fn try_uninhibit(&mut self) -> Result<(), Box<dyn Error>> {
        let proxy = self.conn.with_proxy("org.freedesktop.ScreenSaver", "/ScreenSaver",
                                         Duration::from_secs(1));
        let cookie: u32 = self.cookie.ok_or_else(|| "cookie was None")?;
        let _: () = proxy.method_call("org.freedesktop.ScreenSaver", "UnInhibit",
                                      (cookie, ))?;
        self.cookie = None;
        Ok(())
    }
}

impl SleepInhibitor for DBusDisplaySleepInhibitor {
    fn inhibit(&mut self, application: &str, reason: &str) {
        self.try_inhibit(application, reason).unwrap()
    }

    fn uninhibit(&mut self) {
        self.try_uninhibit().unwrap()
    }
}
