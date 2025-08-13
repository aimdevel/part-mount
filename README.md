# Part Mount
This is mount tool for image files with several partition.  
This tool enable to ease to mount desired partition of the image file.  

## usage
```
Usage: part-mount --partition-number <PARTITION_NUMBER> <DEVICE> <COMMAND>

Commands:
  mount   Mount an existing partition
  format  Format a partition by zeroing its contents and creating a filesystem
  dump    Dump an existing partition
  insert  Insert a file to an existing partition
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <DEVICE>  Target device file

Options:
  -p, --partition-number <PARTITION_NUMBER>  Partition number
  -h, --help                                 Print help
  -V, --version                              Print version
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
