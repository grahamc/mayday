use crate::servers::Device;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
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
            info!(self.logger, "Server gone, killing tailer"; "server" => ?device);
            self.children
                .remove(&device)
                .map(|mut child| child.kill().unwrap());
        }
        for device in added.into_iter().take(5) {
            let sos = device.sos();
            let filename = self
                .output_dir
                .as_path()
                .join(format!("{}-{}", device.hostname, device.id));
            info!(self.logger, "New server, spawning tailer"; "file" => ?filename, "server" => ?device);
            let file = File::create(filename).expect("Failed to create tailer file");
            self.children.insert(
                device,
                Command::new("ssh")
                    .arg("-t")
                    .arg("-t")
                    .arg(&sos)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::from(file))
                    .stderr(Stdio::inherit())
                    .spawn()
                    .expect("Failed to spawn SSH"),
            );
        }
    }
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
