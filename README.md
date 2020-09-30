caffeinate
==========

Rust clone of `caffeinate(8)` from macOS for Linux.

caffeinate uses the `org.freedesktop.login1.Manager.Inhibit` D-Bus method to
prevent idle sleep, as well as the `org.freedesktop.ScreenSaver.Inhibit` D-Bus
method to prevent display sleep. Most desktop environments should support these
methods.
