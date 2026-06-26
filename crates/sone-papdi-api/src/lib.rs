//! Subscribable API: Hyprwire IPC server, D-Bus server, and CLI for sone-papdi.
//!
//! # Architecture
//!
//! The API layer provides external access to shell events and commands:
//!
//! - **Hyprwire server** (`hyprwire_server`): Internal IPC transport using the
//!   hyprwire protocol. All first-party processes (daemon, services, UI) use
//!   this channel for subscribing to events and sending commands.
//! - **D-Bus server** (`dbus_server`): Session-bus service exposing
//!   `org.sone_papdi.Shell1` with methods and signals for external tools.
//! - **CLI** (`sone-papdi` binary): User-facing command-line tool that connects
//!   via hyprwire for internal commands and D-Bus for external scripting.
//!
//! Hyprwire is the primary IPC transport. D-Bus is exposed alongside for
//! interoperability with non-hyprwire tools.

pub mod dbus_server;
pub mod hyprwire_server;
pub mod unix_server;
