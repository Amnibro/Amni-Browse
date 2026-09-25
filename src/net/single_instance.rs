use std::path::PathBuf;
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleInstanceMessage {
    pub url: Option<String>,
    pub private: bool,
    #[serde(default)]
    pub new_window: bool,
}

pub enum SingleInstance {
    Primary(SingleInstanceListener),
    Forwarded,
    Disabled,
}

pub struct SingleInstanceListener {
    path: PathBuf,
    #[cfg(unix)]
    listener: Option<std::os::unix::net::UnixListener>,
}

pub struct SingleInstanceGuard {
    path: PathBuf,
}

impl Drop for SingleInstanceListener {
    fn drop(&mut self) {
        #[cfg(unix)]
        if self.listener.is_some() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        let _ = std::fs::remove_file(&self.path);
    }
}

fn socket_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).or_else(dirs::runtime_dir) {
        dir.join("amni-browse.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
        std::env::temp_dir().join(format!("amni-browse-{}.sock", user))
    }
}

pub fn cleanup() {
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(socket_path());
    }
}

pub fn init() -> SingleInstance {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--new-instance") {
        return SingleInstance::Disabled;
    }

    #[cfg(unix)]
    {
        let sock_path = socket_path();
        let private = args.iter().any(|a| a == "-p" || a == "--private" || a == "--incognito");
        let url = args.iter().skip(1).find(|a| !a.starts_with('-')).cloned();
        let outgoing = SingleInstanceMessage { url, private, new_window: args.iter().any(|a| a == "--new-window") };

        // Attempt to connect to a running instance
        match std::os::unix::net::UnixStream::connect(&sock_path) {
            Ok(mut stream) => {
                use std::io::{Read, Write};
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));
                if let Ok(payload) = serde_json::to_string(&outgoing) {
                    if stream.write_all(payload.as_bytes()).is_ok() {
                        let _ = stream.shutdown(std::net::Shutdown::Write);
                        let mut ack = [0u8; 1];
                        let _ = stream.read(&mut ack);
                        info!("Forwarded request to running instance: {:?}", outgoing);
                        return SingleInstance::Forwarded;
                    }
                }
            }
            Err(_) => {
                // Connection failed. If socket file exists, it is stale from a prior crash or termination.
                if sock_path.exists() {
                    let _ = std::fs::remove_file(&sock_path);
                }
            }
        }

        match std::os::unix::net::UnixListener::bind(&sock_path) {
            Ok(listener) => SingleInstance::Primary(SingleInstanceListener {
                path: sock_path,
                listener: Some(listener),
            }),
            Err(e) => {
                warn!("Could not bind single instance socket {:?}: {}", sock_path, e);
                SingleInstance::Disabled
            }
        }
    }

    #[cfg(not(unix))]
    {
        SingleInstance::Disabled
    }
}

#[cfg(feature = "webview")]
impl SingleInstanceListener {
    pub fn listen(
        mut self,
        proxy: tao::event_loop::EventLoopProxy<()>,
        tx: std::sync::mpsc::Sender<SingleInstanceMessage>,
    ) -> SingleInstanceGuard {
        let path = self.path.clone();
        #[cfg(unix)]
        if let Some(listener) = self.listener.take() {
            std::thread::Builder::new()
                .name("amni-browse-ipc".into())
                .spawn(move || {
                    while let Ok((mut stream, _)) = listener.accept() {
                        use std::io::{Read, Write};
                        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
                        let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));
                        let mut buf = String::new();
                        let _ = stream.read_to_string(&mut buf);
                        let _ = stream.write_all(b"1");
                        let _ = stream.flush();

                        let msg = if let Ok(m) = serde_json::from_str::<SingleInstanceMessage>(&buf) {
                            m
                        } else {
                            let trimmed = buf.trim();
                            SingleInstanceMessage {
                                url: if trimmed.is_empty() { None } else { Some(trimmed.to_string()) },
                                private: false,
                                new_window: false,
                            }
                        };

                        let _ = tx.send(msg);
                        let _ = proxy.send_event(());
                    }
                })
                .expect("single instance listener thread");
        }
        std::mem::forget(self);
        SingleInstanceGuard { path }
    }
}
