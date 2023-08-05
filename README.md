# Part Mount
This is mount tool for image files with several partition.  
This tool enable to ease to mount desired partition of the image file.  

## usage
```
$ sudo part-mount -p <partition number> <device name> <mount dir>
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

