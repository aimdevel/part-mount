# Part Mount
This is mount tool for image files with several partition.  
This tool enable to ease to mount desired partition of the image file.  

## usage
```
A tool to manipulate SD card dump files

Usage: part-mount --partition-number <PARTITION_NUMBER> <DEVICE> <COMMAND>

Commands:
  mount   Mount an existing partition
  format  Format a partition by zeroing its contents and creating a filesystem
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <DEVICE>  Target device file

Options:
  -p, --partition-number <PARTITION_NUMBER>  Partition number
  -h, --help                                 Print help
  -V, --version                              Print version
koma@hides-pc:~/work/part-mount$ ./target/debug/part-mount mount
error: the following required arguments were not provided:
  <MOUNTPOINT>

Usage: part-mount --partition-number <PARTITION_NUMBER> <DEVICE> mount <MOUNTPOINT>
```

## build 
1. install dependencies.
```
$ sudo apt install libparted-dev
```

2. clone this repository.  
```
$ git clone https://github.com/aimdevel/part-mount.git
```

3. build  .
```
$ cd part-mount
$ cargo build
```
There is build result as "part-mount" in `target/debug` directory.
