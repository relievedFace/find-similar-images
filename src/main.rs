use clap::{App, Arg};
use find_similar_images::image_info::ImageInfo;
use find_similar_images::similarity::distance;
use find_similar_images::similarity::CalcHash;
use image;
use image::FilterType;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::fs;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("find  images")
        .version("0.1.0")
        .author("relievedFace <relievedFace@gmaile.com>")
        .about("find similarity images")
        .arg(
            Arg::with_name("width")
                .help(" default value: 8")
                .short("w")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .help("default value: 8")
                .short("h")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("threshold")
                .help("default value 4")
                .short("t")
                .takes_value(true),
        );

    let matches = app.get_matches();
    let width = matches.value_of("width").unwrap_or("8").parse()?;
    let height = matches.value_of("height").unwrap_or("8").parse()?;
    let threshold = matches.value_of("threshold").unwrap_or("4").parse()?;

    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let paths: Vec<_> = reader.lines().flatten().collect();

    let mut images_info = vec![ImageInfo::default(); paths.len()];
    paths
        .par_iter()
        .map(|path| {
            let hash = image::open(&path)
                .map(|image| {
                    image
                        .resize_exact(width, height, FilterType::Lanczos3)
                        .grayscale()
                        .calc_hash()
                })
                .ok();
            let modified = fs::metadata(path).map_or(None, |meta| meta.modified().ok());
            ImageInfo {
                path,
                hash,
                modified,
            }
        })
        .collect_into_vec(&mut images_info);

    let mut similaritys = vec![];
    let mut checked = vec![false; paths.len()];

    for i in (0..paths.len()).rev() {
        if !checked[i] {
            let mut similarity = vec![images_info[i]];
            checked[i] = true;
            for j in 0..i {
                match (images_info[i].hash, images_info[j].hash, checked[j]) {
                    (Some(hash1), Some(hash2), false) => {
                        if distance(hash1, hash2, width * height) < threshold {
                            similarity.push(images_info[j]);
                            checked[j] = true;
                        }
                    }
                    _ => continue,
                }
            }
            similarity.sort_by_key(|k| Reverse(k.modified));
            similaritys.push(similarity);
        }
    }

    similaritys.sort_by_key(|k| k[0].modified);

    for s in similaritys.iter().filter(|s| s.len() > 1) {
        writeln!(
            writer,
            "{}",
            s.iter()
                .map(|x| x.path.to_string().replace(" ", "\\ "))
                .collect::<Vec<_>>()
                .join(" "),
        )?;
    }
    Ok(())
}
