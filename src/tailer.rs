use crate::servers::Device;
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

pub struct Tailer {
    logger: slog::Logger,
    output_dir: PathBuf,
    children: HashMap<Device, Child>,
}

impl Tailer {
    pub fn new(logger: slog::Logger, output_dir: PathBuf) -> Tailer {
        Tailer {
            logger,
            output_dir,
            children: HashMap::new(),
        }
    }

    pub fn update(&mut self, added: HashSet<Device>, removed: HashSet<Device>) {
        for device in removed.into_iter() {
            if let Some(mut child) = self.children.remove(&device) {
                info!(self.logger, "Server gone, killing tailer"; "server" => ?device);
                if child.kill().is_err() {
                    info!(self.logger, "Tailer already dead."; "server" => ?device);
                }
            }
        }

        let output_dir = self.output_dir.as_path();
        for (device, child) in self.children.iter_mut() {
            let logger = self.logger.new(o!(
                "id" => format!("{}", device.id),
                "hostname" => format!("{}", device.hostname)
            ));
            match child.try_wait() {
                Ok(Some(status)) => {
                    info!(logger, "Tailer exited, respawning"; "status" => ?status);

                    *child = spawn_tailer(&output_dir, &device).expect("Failed to re-spawn tailer");
                }
                Ok(None) => {
                    // still running
                    if let Some(stdin) = child.stdin.as_mut() {
                        let backspace = 0x08;
                        if let Err(e) = (*stdin).write(&[backspace]) {
                            error!(self.logger, "Failed to write to child's stdin"; "e" => ?e);
                        }
                    } else {
                        error!(self.logger, "bug! child has no stdin?");
                    }
                }
                Err(e) => {
                    error!(self.logger, "Error waiting on tailer"; "e" => ?e);
                }
            }
        }

        for device in added.into_iter() {
            info!(self.logger, "New server, spawning tailer"; "server" => ?device);
            let child =
                spawn_tailer(&self.output_dir.as_path(), &device).expect("Failed to spawn tailer");

            self.children.insert(device, child);
        }
    }
}

fn spawn_tailer(output_dir: &Path, device: &Device) -> Result<Child, std::io::Error> {
    let sos = device.sos();
    let filename = output_dir.join(format!("{}-{}", device.hostname, device.id));

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)?;

    file.write_all(b"\n\n ~~~ MAYDAY TAILER RESTART ~~~ \n\n")?;

    std::thread::sleep(std::time::Duration::from_millis(50));
    Command::new("ssh")
        .arg("-t")
        .arg("-t")
        .arg(&sos)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(file))
        .stderr(Stdio::inherit())
        .spawn()
}

impl Drop for Tailer {
    fn drop(&mut self) {
        let logger = self.logger.new(o!());
        for (device, child) in self.children.iter_mut() {
            info!(logger, "Cleaning up tailer"; "device" => ?device);
            child.kill().expect("Failed to kill child");
        }
    }
}
