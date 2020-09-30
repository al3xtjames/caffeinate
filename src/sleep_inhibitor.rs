pub trait SleepInhibitor {
    fn inhibit(&mut self, application: &str, reason: &str);
    fn uninhibit(&mut self);
}

#[cfg(target_os = "linux")]
pub mod platform {
    use crate::dbus_sleep_inhibitor::{DBusSystemSleepInhibitor, DBusDisplaySleepInhibitor};
    pub type SystemSleepInhibitor = DBusSystemSleepInhibitor;
    pub type DisplaySleepInhibitor = DBusDisplaySleepInhibitor;
}
