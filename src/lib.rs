extern crate libparted;
extern crate sys_mount;
extern crate loopdev;

use std::fs::OpenOptions;
use std::io::{Write, Seek, SeekFrom, Read};
use std::process::Command;

use libparted::{Device, Disk};
use sys_mount::Mount;
use loopdev::LoopDevice;

pub struct PartMount {
    pub device: String,
    pub partition_number: i32,
    // Optional test partition info: (offset, length)
    pub test_partition_info: Option<(u64, u64)>,
}

impl PartMount {
    /// Retrieve the offset and length (in bytes) of the specified partition.
    fn get_partition_info(&self) -> Option<(u64, u64)> {
        if let Some(info) = self.test_partition_info {
            return Some(info);
        }
        let mut dev = match Device::get(&self.device) {
            Ok(dev) => dev,
            Err(why) => {
                eprintln!("unable to get {} device: {}", &self.device, why);
                return None;
            }
        };
        let sector_size = dev.sector_size();
        let disk = match Disk::new(&mut dev) {
            Ok(disk) => disk,
            Err(why) => {
                eprintln!("unable to create disk object: {}", why);
                return None;
            }
        };
        for part in disk.parts() {
            if part.num() == self.partition_number {
                let offset = part.geom_start() as u64 * sector_size;
                // Assuming libparted provides geom_length() for partition size.
                let length = part.geom_length() as u64 * sector_size;
                return Some((offset, length));
            }
        }
        eprintln!("partition {} is not found.", self.partition_number);
        None
    }

    /// Mount the specified partition.
    pub fn mount(&self, mount_point: &str) {
        let (offset, _length) = match self.get_partition_info() {
            Some(info) => info,
            None => {
                eprintln!("cannot get partition info");
                return;
            }
        };
        if offset == 0 {
            return;
        }
        // In test mode, simulate mounting without performing actual mount.
        if cfg!(test) {
            println!("[Test Mode] Simulating mount of {} at {}", self.device, mount_point);
            return;
        }
        let mount_builder = Mount::builder().loopback_offset(offset);
        let mount_result = mount_builder.mount(&self.device, mount_point);
        match mount_result {
            Ok(mount) => {
                println!("mount success");
                match mount.backing_loop_device() {
                    Some(path) => {
                        println!("setting auto-clear flag on {}.", &path.display());
                        let ld = LoopDevice::open(path).unwrap();
                        ld.detach().unwrap();
                    }
                    None => println!("cannot get loopback device name."),
                };
            }
            Err(why) => eprintln!("error!: {}", why),
        };
    }

    /// Format the specified partition by zeroing out its contents and creating a filesystem.
    /// Supported filesystem types are "vfat" and "ext4".
    pub fn format_partition(&self, fs_type: String) {
        let (offset, length) = match self.get_partition_info() {
            Some(info) => info,
            None => {
                eprintln!("cannot get partition info");
                return;
            }
        };
        println!("Formatting partition: offset = {}, length = {} bytes", offset, length);

        let mut file = match OpenOptions::new().read(true).write(true).open(&self.device) {
            Ok(f) => f,
            Err(why) => {
                eprintln!("failed to open device {}: {}", self.device, why);
                return;
            }
        };

        if let Err(why) = file.seek(SeekFrom::Start(offset)) {
            eprintln!("seek error: {}", why);
            return;
        }

        // Write zeros in chunks.
        let chunk_size: usize = 4096;
        let zeros = vec![0u8; chunk_size];
        let mut bytes_remaining = length;
        while bytes_remaining > 0 {
            let write_size = if bytes_remaining < chunk_size as u64 {
                bytes_remaining as usize
            } else {
                chunk_size
            };
            if let Err(why) = file.write_all(&zeros[..write_size]) {
                eprintln!("write error: {}", why);
                return;
            }
            bytes_remaining -= write_size as u64;
        }
        println!("Zeroing complete.");

        // Validate and process the filesystem type.
        let fs = fs_type.to_lowercase();
        if fs != "vfat" && fs != "ext4" {
            eprintln!("Unsupported filesystem type: {}. Supported types: vfat, ext4", fs_type);
            return;
        }

        // For safety in tests, simulate formatting if in test configuration.
        if cfg!(test) {
            println!("[Test Mode] Simulating creation of {} filesystem.", fs);
            println!("Formatting complete.");
            return;
        }

        // Create a loop device for the partition using losetup.
        // Command: losetup --find --show -o <offset> <device>
        let output = Command::new("losetup")
            .arg("--find")
            .arg("--show")
            .arg("-o")
            .arg(offset.to_string())
            .arg(&self.device)
            .output();

        let loop_device = match output {
            Ok(out) if out.status.success() => {
                let dev = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if dev.is_empty() {
                    eprintln!("Failed to obtain loop device.");
                    return;
                }
                dev
            },
            Ok(out) => {
                eprintln!("losetup error: {}", String::from_utf8_lossy(&out.stderr));
                return;
            },
            Err(why) => {
                eprintln!("failed to execute losetup: {}", why);
                return;
            }
        };

        println!("Loop device {} created for formatting.", loop_device);

        // Determine mkfs command based on filesystem type.
        let mkfs_cmd = if fs == "vfat" { "mkfs.vfat" } else { "mkfs.ext4" };
        let mkfs_args = if fs == "vfat" {
            vec!["-F", "32", &loop_device]
        } else {
            vec!["-F", &loop_device]
        };

        let output = Command::new(mkfs_cmd)
            .args(&mkfs_args)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                println!("{} filesystem created on {}.", fs, loop_device);
            },
            Ok(out) => {
                eprintln!("{} error: {}", mkfs_cmd, String::from_utf8_lossy(&out.stderr));
                // Attempt to detach loop device before returning.
                let _ = Command::new("losetup").arg("-d").arg(&loop_device).output();
                return;
            },
            Err(why) => {
                eprintln!("failed to execute {}: {}", mkfs_cmd, why);
                let _ = Command::new("losetup").arg("-d").arg(&loop_device).output();
                return;
            }
        }

        // Detach the loop device.
        let output = Command::new("losetup")
            .arg("-d")
            .arg(&loop_device)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                println!("Loop device {} detached.", loop_device);
            },
            Ok(out) => {
                eprintln!("Error detaching loop device {}: {}", loop_device, String::from_utf8_lossy(&out.stderr));
            },
            Err(why) => {
                eprintln!("failed to execute losetup detach: {}", why);
            }
        }

        println!("Formatting complete.");
    }

    /// dump one partition from sd card image file.
    pub fn dump_partition(&self, output_path: &str) {
        let (offset, length) = match self.get_partition_info() {
            Some(info) => info,
            None => {
                eprintln!("cannot get partition info");
                return;
            }
        };
        println!("Dumping partition: offset = {}, length = {}", offset, length);

        let mut file = match OpenOptions::new().read(true).open(&self.device) {
            Ok(f) => f,
            Err(why) => {
                eprintln!("failed to open device {}: {}", self.device, why);
                return;
            }
        };

        if let Err(why) = file.seek(SeekFrom::Start(offset)) {
            eprintln!("seek error: {}", why);
            return;
        }

        let mut output_file = match OpenOptions::new().
            create(true).truncate(true).write(true).open(&output_path) {
            Ok(f) => f,
            Err(why) => {
                eprintln!("file create error: {} : {}", &output_path, why);
                return;
            }
        };
        
        let mut bytes_remaining = length;
        let mut buffer = [0u8; 8192];
        while bytes_remaining > 0 {
            let read_size = if bytes_remaining < buffer.len() as u64 {
                bytes_remaining as usize
            } else {
                buffer.len()
            };
            match file.read(&mut buffer[..read_size]) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(why) = output_file.write_all(&buffer[..n]) {
                        eprintln!("write error: {}", why);
                        return;
                    }
                    bytes_remaining -= n as u64;
                },
                Err(why) => {
                    eprintln!("read error: {}", why);
                    return;
                }
            }
        }
        println!("Dump complete.");
    }


    /// insert binary file to 
    pub fn insert_partition(&self, file_name: String) {
        
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::PathBuf;

    fn create_temp_file_with_data(file_name: &str, size: usize, value: u8) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(file_name);
        let mut file = File::create(&path).unwrap();
        let data = vec![value; size];
        file.write_all(&data).unwrap();
        path
    }

    #[test]
    fn test_format_partition_without_fs() {
        // Create a temporary file with non-zero content.
        let file_size: usize = 16384; // 16 KB
        let temp_file_path 
            = create_temp_file_with_data("format_partition_without_fs", file_size, 0xFF);

        let part = PartMount {
            device: temp_file_path.to_string_lossy().to_string(),
            partition_number: 1,
            test_partition_info: Some((0, file_size as u64)),
        };

        // Here, we avoid actual filesystem formatting in tests.
        part.format_partition("ext4".to_string());

        // Verify that the entire file content is zero.
        let mut file = File::open(&temp_file_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();

        // Clean up the temporary file.
        fs::remove_file(&temp_file_path).unwrap();

        assert!(content.iter().all(|&b| b == 0), "Not all bytes were zeroed");
    }

    #[test]
    fn test_format_partition_with_fs() {
        // Create a temporary file with non-zero content.
        let file_size: usize = 16384; // 16 KB
        let temp_file_path
            = create_temp_file_with_data("format_partition_with_fs", file_size, 0xFF);

        let part = PartMount {
            device: temp_file_path.to_string_lossy().to_string(),
            partition_number: 1,
            test_partition_info: Some((0, file_size as u64)),
        };

        // Test with vfat; in test mode, actual formatting is simulated.
        part.format_partition("vfat".to_string());

        let mut file = File::open(&temp_file_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();

        fs::remove_file(&temp_file_path).unwrap();
        assert!(content.iter().all(|&b| b == 0), "Not all bytes were zeroed");
    }

    #[test]
    fn test_mount_subcommand() {
        // Create a temporary file with dummy content.
        let file_size: usize = 8192; // 8 KB
        let temp_file_path = 
            create_temp_file_with_data("mount_subcommand", file_size, 0xAA);

        let part = PartMount {
            device: temp_file_path.to_string_lossy().to_string(),
            partition_number: 1,
            test_partition_info: Some((0, file_size as u64)),
        };

        // In test mode, mount() should simulate mounting.
        part.mount("dummy_mountpoint");

        // Clean up the temporary file.
        fs::remove_file(&temp_file_path).unwrap();
    }
}
