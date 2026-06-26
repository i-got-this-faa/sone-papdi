//! sone-papdi — Command-line interface for the sone-papdi shell.
//!
//! Connects to the daemon via hyprwire IPC to subscribe to events,
//! query state, and send imperative commands.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hyprwire::Dispatch;
use sone_papdi_core::ipc::{IpcMessage, IpcResponse, ShellCommand};
use sone_papdi_core::paths::daemon_socket;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Generated protocol bindings.
#[allow(clippy::enum_variant_names)]
mod shell_proto {
    hyprwire::include_protocol!("sone_papdi_shell_v1");
}

use shell_proto::client::shell_manager_v1::ShellManagerV1;

// ── CLI definition ───────────────────────────────────────────

#[derive(Parser)]
#[command(name = "sone-papdi", about = "sone-papdi shell CLI")]
struct Cli {
    #[arg(short = 's', long = "socket", global = true)]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Subscribe { topics: Vec<String> },
    #[command(name = "clipboard-list")]
    ClipboardList { limit: Option<usize> },
    #[command(name = "clipboard-copy")]
    ClipboardCopy { id: i64 },
    #[command(name = "clipboard-delete")]
    ClipboardDelete { id: i64 },
    #[command(name = "clipboard-search")]
    ClipboardSearch { query: String },
    #[command(name = "clipboard-clear")]
    ClipboardClear,
    #[command(name = "clipboard-pin")]
    ClipboardPin { id: i64, pinned: bool },
    ShowLauncher,
    HideLauncher,
    #[command(name = "set-volume")]
    SetVolume { volume: f64, sink: Option<String> },
    #[command(name = "toggle-mute")]
    ToggleMute { sink: Option<String> },
    #[command(name = "set-wallpaper")]
    SetWallpaper { path: String, output: Option<String> },
    #[command(name = "set-theme")]
    SetTheme { name: String },
    #[command(name = "toggle-dark-mode")]
    ToggleDarkMode,
    #[command(name = "switch-workspace")]
    SwitchWorkspace { workspace: String },
    #[command(name = "show-notification-center")]
    ShowNotificationCenter,
    Lock,
    Suspend,
    Logout,
    Reload,
}

// ── Client state ─────────────────────────────────────────────

struct CliState {
    event_tx: mpsc::Sender<String>,
}

impl Dispatch<ShellManagerV1> for CliState {
    fn event(
        &mut self,
        _object: &ShellManagerV1,
        event: <ShellManagerV1 as hyprwire::Object>::Event<'_>,
    ) {
        match event {
            shell_proto::client::shell_manager_v1::Event::Event { payload, .. } => {
                let _ = self.event_tx.send(payload);
            }
            shell_proto::client::shell_manager_v1::Event::Response { payload } => {
                let _ = self.event_tx.send(payload);
            }
            shell_proto::client::shell_manager_v1::Event::Pong => {}
        }
    }
}

// ── Connection helper ────────────────────────────────────────

fn connect(
    socket: &PathBuf,
) -> Result<(
    ShellManagerV1,
    mpsc::Receiver<String>,
    thread::JoinHandle<()>,
)> {
    let client = hyprwire::client::Client::connect(socket)
        .with_context(|| format!("connect to {}", socket.display()))?;
    let eq = client.new_event_queue();
    let (event_tx, event_rx) = mpsc::channel();
    let mut state = CliState { event_tx };
    eq.wait_for_handshake(&mut state)?;
    let manager: ShellManagerV1 = client.bind(&eq, &mut state, 1)?;
    let handle = thread::spawn(move || loop {
        if let Err(e) = eq.dispatch_events(&mut state, true) {
            eprintln!("dispatch error: {e}");
            thread::sleep(Duration::from_millis(100));
        }
    });
    Ok((manager, event_rx, handle))
}

// ── Main ─────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();
    let socket = cli.socket.clone().unwrap_or_else(daemon_socket);
    if !socket.exists() {
        eprintln!("daemon socket not found at {}", socket.display());
        eprintln!("is sone-papdi running?");
        std::process::exit(1);
    }
    match &cli.command {
        Commands::Subscribe { topics } => subscribe(&socket, topics),
        _ => send_command(&socket, cli.command_to_shell_cmd()),
    }
}

fn subscribe(socket: &PathBuf, topics: &[String]) -> Result<()> {
    let (manager, rx, _handle) = connect(socket)?;
    for topic in topics {
        manager.send_subscribe(topic);
    }
    while let Ok(payload) = rx.recv() {
        println!("{payload}");
    }
    Ok(())
}

fn send_command(socket: &PathBuf, cmd: ShellCommand) -> Result<()> {
    let (manager, rx, _handle) = connect(socket)?;
    let payload = serde_json::to_string(&IpcMessage::Command(cmd))?;
    manager.send_command(&payload);
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(json_str) => {
            if let Ok(resp) = serde_json::from_str::<IpcResponse>(&json_str) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).unwrap_or_default()
                );
            } else {
                println!("{json_str}");
            }
            Ok(())
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("timed out waiting for response");
            std::process::exit(1);
        }
        Err(_) => Ok(()),
    }
}

impl Cli {
    fn command_to_shell_cmd(&self) -> ShellCommand {
        match &self.command {
            Commands::ClipboardList { limit } => ShellCommand::ClipboardList { limit: *limit },
            Commands::ClipboardCopy { id } => ShellCommand::ClipboardCopy { id: *id },
            Commands::ClipboardDelete { id } => ShellCommand::ClipboardDelete { id: *id },
            Commands::ClipboardSearch { query } => {
                ShellCommand::ClipboardSearch { query: query.clone() }
            }
            Commands::ClipboardClear => ShellCommand::ClipboardClear,
            Commands::ClipboardPin { id, pinned } => {
                ShellCommand::ClipboardPin { id: *id, pinned: *pinned }
            }
            Commands::ShowLauncher => ShellCommand::ShowLauncher,
            Commands::HideLauncher => ShellCommand::HideLauncher,
            Commands::SetVolume { volume, sink } => ShellCommand::SetVolume {
                sink: sink.clone(),
                volume: *volume,
            },
            Commands::ToggleMute { sink } => ShellCommand::ToggleMute { sink: sink.clone() },
            Commands::SetWallpaper { path, output } => ShellCommand::SetWallpaper {
                path: path.clone(),
                output: output.clone(),
            },
            Commands::SetTheme { name } => ShellCommand::SetTheme { name: name.clone() },
            Commands::ToggleDarkMode => ShellCommand::ToggleDarkMode,
            Commands::SwitchWorkspace { workspace } => {
                ShellCommand::SwitchWorkspace {
                    workspace: workspace.clone(),
                }
            }
            Commands::ShowNotificationCenter => ShellCommand::ShowNotificationCenter,
            Commands::Lock => ShellCommand::LockScreen,
            Commands::Suspend => ShellCommand::Suspend,
            Commands::Logout => ShellCommand::Logout,
            Commands::Reload => ShellCommand::Reload,
            Commands::Subscribe { .. } => unreachable!(),
        }
    }
}
