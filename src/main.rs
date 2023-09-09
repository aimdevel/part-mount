extern crate libparted;
extern crate sys_mount;
extern crate loopdev;

use std::process::exit;
use clap::Parser;
use libparted::{Device, Disk};
use sys_mount::{
    Mount,
};
use loopdev::LoopDevice;

#[derive(Parser)]
struct Args {
    /// Target device name.
    device: String,
    /// Partition number
    #[arg(short)]
    partition_number: i32,
    /// Mount point
    mountdir: String,
}

struct PartMount{
    device: String,
    mount_point: String,
    partition_number: i32,
}

impl PartMount{
    fn get_offset(&self) -> Option<u64> {
        let mut dev = match Device::get(&self.device){
            Ok(dev) => dev,
            Err(why) => {
                eprintln!("unable to get {} device: {}", &self.device,why);
                return None;
            }
        };
        let sector_size = dev.sector_size();
        let disk = match Disk::new(&mut dev){
            Ok(disk) => disk,
            Err(why) => {
                eprintln!("unable to create disk object: {}", why);
                return None;
            }
        };
        for part in disk.parts() {
            if part.num() == self.partition_number{
                let offset = part.geom_start() as u64 * sector_size;
                return Some(offset);
            }
        }
        eprintln!("partition {} is not found.", self.partition_number);
        return None;
    }

    pub fn mount(&self) {
        let offset = match self.get_offset() {
            Some(x) => x,
            None => {
                eprintln!("cannot get partition's offset");
                0
            },
        };
        if offset == 0 {
            return;
        }
        let mount_builder = Mount::builder()
            .loopback_offset(offset);

        let mount_result = mount_builder.mount(&self.device, &self.mount_point);

        match mount_result {
            Ok(mount) => {
                println!("mount succcess");
                match mount.backing_loop_device() {
                    Some(path) => {
                        println!("make auto clear flag of {} on.", &path.display());
                        let ld = LoopDevice::open(path).unwrap();
                        ld.detach().unwrap();
                    },
                    None => println!("cannot get loopback device name."),
                };
            },
            Err(why) => eprintln!("error!:{}", why),
        };
    }
}

fn main(){
    let args = Args::parse();

    if args.partition_number < 1 {
        eprintln!("invalid partition number:{}.", args.partition_number);
        exit(1);
    }

    let mount = PartMount{
        device: args.device,
        mount_point: args.mountdir,
        partition_number: args.partition_number,
    };
    mount.mount();
}