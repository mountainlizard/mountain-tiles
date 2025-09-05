use std::{
    env,
    io::{self, BufReader},
    sync::mpsc,
    thread,
};

use camino::Utf8PathBuf;
use interprocess::local_socket::ListenerOptions;
use log::error;
use single_instance::SingleInstance;
use std::sync::OnceLock;
use {
    interprocess::local_socket::{prelude::*, GenericNamespaced, Stream},
    std::io::prelude::*,
};

static SINGLE_INSTANCE_CELL: OnceLock<Option<SingleInstance>> = OnceLock::new();

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum IpcMessage {
    FileOpen { filepath: Utf8PathBuf },
}

pub struct IpcListener {
    _thread: thread::JoinHandle<()>,
    receiver: mpsc::Receiver<IpcMessage>,
}

impl IpcListener {
    // Attempt to receive a message without blocking, returning None if none is available
    pub fn poll_recv(&mut self) -> Option<IpcMessage> {
        self.receiver.try_recv().ok()
    }
}

/// Return true if this is known to be the first (only) instance of the application
/// running, false if there may be an instance already running (either because this
/// has been confirmed, or because it was not possible to check).
fn application_is_first_instance(uniq_id: &str) -> bool {
    // The first time this is called, we try to create a `SingleInstance`.
    // If this succeeds, we use it to check whether this application is in
    // fact the only instance running. We then store the instance as Some in the
    // `OnceLock`.
    // If we can't create the `SingleInstance`, or we can but it indicates we are
    // NOT the only instance running, we store `None` in the OnceLock.
    // This achieves two things:
    // 1. If this is the single instance of this application, the `OnceLock` retains
    //    the `SingleInstance` until the program shuts down, as required by the
    //    `single-instance` crate.
    // 2. If the `OnceLock` value is Some, then we're the only application instance,
    //    and we can return true. If it's None, then either we're not the only application
    //    instance, or we can't tell because of an error, so we return false for safety.
    SINGLE_INSTANCE_CELL
        .get_or_init(|| match SingleInstance::new(uniq_id) {
            Ok(instance) => {
                if instance.is_single() {
                    Some(instance)
                } else {
                    None
                }
            }
            Err(e) => {
                error!(
                    "Failed to check whether this is the only instance of the application: {}",
                    e
                );
                None
            }
        })
        .is_some()
}

// Define a function that checks for errors in incoming connections. We'll use this to filter
// through connections that fail on initialization for one reason or another.
fn handle_listener_error(conn: io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            log::warn!("Incoming connection failed: {e}");
            None
        }
    }
}

pub fn create_ipc_listener(uniq_id: &str, ctx: egui::Context) -> eyre::Result<IpcListener> {
    let name_string = format!("{}.interprocess", uniq_id);
    let (sender, receiver) = mpsc::channel();
    let name = name_string.to_ns_name::<GenericNamespaced>()?;
    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_sync()?;

    let thread = thread::spawn(move || {
        for conn in listener.incoming().filter_map(handle_listener_error) {
            let thread_sender = sender.clone();
            let thread_ctx = ctx.clone();
            thread::spawn(move || {
                let mut reader = BufReader::new(conn);
                let mut buffer = String::with_capacity(256);
                if reader.read_line(&mut buffer).is_ok() {
                    log::trace!("Received message: {}", buffer);
                    match serde_json::from_str(&buffer) {
                        Ok(message) => {
                            // Note we ignore any error - it just indicates the receiver has
                            // been dropped, in which case any messages are no longer needed
                            if thread_sender.send(message).is_ok() {
                                // Make sure the UI draws another frame, the update
                                // method will then poll the channel and receive the
                                // message. Otherwise, the message will only be handled
                                // when the UI needs to redraw for some other reason, e.g.
                                // mouse movement.
                                thread_ctx.request_repaint();
                                log::trace!("Sent message to mpsc");
                            }
                        }
                        Err(e) => log::warn!("Error decoding message: {}", e),
                    }
                }
            });
        }
    });

    Ok(IpcListener {
        _thread: thread,
        receiver,
    })
}

/// Send a message to the first instance of the application.
fn send_to_first_instance(uniq_id: &str, message: &IpcMessage) -> eyre::Result<()> {
    let name = uniq_id.to_ns_name::<GenericNamespaced>()?;

    // Create our connection. This will block until the server accepts our connection, but will
    // fail immediately if the server hasn't even started yet; somewhat similar to how happens
    // with TCP, where connecting to a port that's not bound to any server will send a "connection
    // refused" response, but that will take twice the ping, the roundtrip time, to reach the
    // client.
    let mut conn = Stream::connect(name)?;

    // Send as a single-line JSON string, newline terminated
    let json = serde_json::to_string(message)?;
    let payload = format!("{}\n", json);

    // Send our message into the stream. This will finish either when the whole message has been
    // sent or if a send operation returns an error.
    conn.write_all(payload.as_bytes())?;

    log::trace!("Sent message {:?} to first instance", message);

    Ok(())
}

/// Perform startup for single instance handling.
/// This will check whether this is the only instance of the application running.
/// If this is the only instance, it will set up a thread to listen for interprocess
/// communication from subsequent instances, to accept filenames to open.
/// If this is NOT the only instance, it will try to send any filename specified
/// as args to the first instance.
///
/// This will return true if this instance is NOT the only instance, in this case
/// you should normally just exit the application.
///
/// Note that this always returns false on macos, and performs no other actions.
/// This is because macOS will already stop us running multiple instances of an
/// application bundle, and uses its own system for opening double-clicked files
/// in an existing instance.
/// TODO: Support opening double clicked files on macos - this is waiting on the next
/// winit version (and an egui release using this) with support for the required
/// event.
pub fn instance_startup(unique_id: &str) -> bool {
    let is_macos = cfg!(target_os = "macos");

    if is_macos {
        return false;
    }

    if application_is_first_instance(format!("{}.single-instance", unique_id).as_str()) {
        log::trace!("First instance.");
        false
    } else {
        log::trace!("Application is already running...");

        // TODO: Handle multiple filenames?
        let args: Vec<String> = env::args().collect();
        let filename = args.get(1);

        if let Some(filename) = filename {
            log::trace!(
                "...received filename {}, sending to main instance",
                filename
            );

            let filepath = Utf8PathBuf::from(filename);

            match send_to_first_instance(
                format!("{}.interprocess", unique_id).as_str(),
                &IpcMessage::FileOpen { filepath },
            ) {
                Ok(_) => log::trace!("...sent"),
                Err(e) => log::error!("Failed to send filename arg to first instance: {}", e),
            }
        }
        log::trace!("...exiting");
        true
    }
}
