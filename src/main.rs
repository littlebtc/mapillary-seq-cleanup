use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs::File, io::Seek};

use anyhow::{ensure, Context};
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;
use clap::Parser;
use geo::{GeodesicDistance, Point};
use serde_json::{json, Value};

const TIME_FORMAT: &str = "%Y_%m_%d_%H_%M_%S_%3f";

#[derive(Parser, Debug)]
struct Cli {
    /// Time zone used to convert timestamps to UTC e.g. Asia/Taipei
    #[arg(long, default_value_t = Tz::UTC, value_parser=timezone_parser)]
    timezone: Tz,

    /// Cut sequence if adjacent images exceeds specified seconds.
    #[arg(long = "cutoff_time", default_value_t = 10)]
    cutoff_time: i64,

    /// Consider following image is a duplicate if distance between two
    /// are lower than that meters.
    #[arg(long = "duplicate_distance", default_value_t = 2.0)]
    duplicate_distance: f64,

    /// The maximum sequence image count.
    #[arg(long="max_sequence_length", default_value_t = 200, value_parser=clap::value_parser!(u64).range(1..))]
    max_sequence_length: u64,

    /// Path to the images and mapillary_image_description.json.
    #[arg(value_hint = clap::ValueHint::DirPath)]
    path: PathBuf,
}

fn timezone_parser(tz: &str) -> Result<Tz, String> {
    tz.parse::<Tz>()
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    ensure!(cli.path.is_dir(), "Input path should be a dir");

    let desc_path = cli.path.join("mapillary_image_description.json");
    let mut desc_fd = File::options()
        .read(true)
        .write(true)
        .open(desc_path)?;
    let mut desc_value: Value = serde_json::from_reader(BufReader::new(&desc_fd))?;

    let images = desc_value.as_array_mut().context("JSON should be lists")?;

    let mut skipped_count: u64 = 0;
    let mut image_count: u64 = 0;

    // Store on previous results
    let mut prev_orig_seq_id: i64 = 0;
    let mut prev_time: Option<DateTime<Tz>> = None;
    let mut prev_point: Option<Point> = None;

    // Store current sequence
    let mut seq_id: u64 = 0;
    let mut seq_len: u64 = 0;

    for entry in images.iter_mut() {
        let image = entry.as_object_mut().context("Entry should be dict")?;
        if image.contains_key("error") {
            continue;
        }
        let orig_seq_id = image["MAPSequenceUUID"]
            .as_str()
            .map(|str| str.parse::<i64>())
            .context("Seq ID parse error")??;

        let point = Point::new(
            image["MAPLongitude"]
                .as_f64()
                .context("coordinates parse error")?,
            image["MAPLatitude"]
                .as_f64()
                .context("coordinates parse error")?,
        );
        let time = cli.timezone.datetime_from_str(
            image["MAPCaptureTime"]
                .as_str()
                .context("Datetime parse error")?,
            TIME_FORMAT,
        )?;
        if let (Some(prev_point), Some(prev_time)) = (prev_point, prev_time) {
            if point.geodesic_distance(&prev_point) < cli.duplicate_distance {
                image.clear();
                skipped_count += 1;
            } else {
                if time.signed_duration_since(prev_time) > Duration::seconds(cli.cutoff_time)
                    || prev_orig_seq_id != orig_seq_id
                    || seq_len >= cli.max_sequence_length
                {
                    seq_id += 1;
                    seq_len = 0;
                }
                image["MAPCaptureTime"] = json!(format!("{}", time.with_timezone(&Utc).format(TIME_FORMAT)));
                image["MAPSequenceUUID"] = json!(seq_id.to_string());
                seq_len += 1;
            }
        }
        prev_orig_seq_id = orig_seq_id;
        prev_point = Some(point);
        prev_time = Some(time);
        image_count += 1;
    }
    images.retain(|entry| {
        return match entry {
            Value::Object(image) => !image.is_empty(),
            _ => false,
        };
    });
    desc_fd.set_len(0)?;
    desc_fd.seek(std::io::SeekFrom::Start(0))?;
    serde_json::to_writer_pretty(BufWriter::new(&desc_fd), &images)?;
    desc_fd.flush()?;
    println!("{image_count} entries proceeded, skipped {skipped_count} entries.");
    println!(
        "Re-written with {0} entries and {1} sequences.",
        images.len(),
        seq_id + 1
    );
    Ok(())
}
