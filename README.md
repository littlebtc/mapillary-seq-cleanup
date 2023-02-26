This is a tool used by the author (littlebtc) in order to upload GoPro Hero 9+ timelapsed images,
which is unable to check duplicates and cutoff sequences because the direction is missing in EXIF.

### Tested with

* `mapillary_tools` v0.10.0
* GoPro Hero 10 (In theory Hero 9+ should work)

### Installation

It is written in Rust and published on crates.io.

[Install Rust](https://www.rust-lang.org/tools/install) and use Cargo to install it:
```sh
cargo install mapillary-seq-cleanup
```

Assume you have installed `mapillary_tools`, for all images stored in `gptodo`, Process the image description file between the process and upload steps:

```sh
mapillary_tools process gptodo --interpolate_directions --skip_process_errors & \
mapillary_seq_cleanup -z "Asia/Taipei" gptodo & \
mapillary_tools upload gptodo
```

### Usage

```
Usage: mapillary_seq_cleanup [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the images and mapillary_image_description.json

Options:
      --timezone <TIMEZONE>
          Time zone used to convert timestamps to UTC e.g. Asia/Taipei [default: UTC]
      --cutoff_time <CUTOFF_TIME>
          Cut sequence if adjacent images exceeds specified seconds [default: 10]
      --duplicate_distance <DUPLICATE_DISTANCE>
          Consider following image is a duplicate if distance between two are lower than that meters [default: 2]
      --max_sequence_length <MAX_SEQUENCE_LENGTH>
          The maximum sequence image count [default: 200]
  -h, --help
          Print help
```

### License

MIT

### See Also

* [The previous approach with Python](https://github.com/littlebtc/mapillary-cleanup)