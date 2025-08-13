use clap::{Parser, Subcommand};
use std::process::exit;

use part_mount::PartMount;

/// A tool to manipulate SD card dump files.
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: CommandOption,
    /// Target device file.
    device: String,
    /// Partition number.
    #[arg(short, long)]
    partition_number: i32,
}

#[derive(Subcommand)]
enum CommandOption {
    /// Mount an existing partition.
    Mount {
        /// Mount point for the partition.
        mountpoint: String,
    },
    /// Format a partition by zeroing its contents and creating a filesystem.
    Format {
        /// Filesystem type: supported values are vfat and ext4.
        #[arg(short, long)]
        fs_type: String,
    },
    /// Dump an existing partition.
    Dump {
        /// Output file name
        output: String,
    }
}

fn main(){
    let cli = Cli::parse();

    if cli.partition_number < 1 {
        eprintln!("invalid partition number: {}.", cli.partition_number);
        exit(1);
    }

    let part = PartMount {
        device: cli.device.clone(),
        partition_number: cli.partition_number,
        test_partition_info: None,
    };

    match cli.command {
        CommandOption::Mount { mountpoint } => {
            part.mount(&mountpoint);
        },
        CommandOption::Format { fs_type } => {
            part.format_partition(fs_type);
        },
        CommandOption::Dump { output } => {
            part.dump_partition(output.as_str());
        },
    }
}