//! Shared types, IPC primitives, and event bus definitions for sone-papdi.

pub mod bus;
pub mod events;
pub mod ipc;
pub mod paths;

pub use bus::EventBus;
pub use events::{event_topic, ShellEvent};
pub use ipc::{recv_message, send_message, IpcMessage, IpcResponse, ShellCommand};
pub use paths::{bar_socket, clipboard_socket, config_dir, daemon_socket, data_dir, runtime_dir};
