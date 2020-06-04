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
                .help("default value 2")
                .short("t")
                .takes_value(true),
        );

    let matches = app.get_matches();
    let width = matches.value_of("width").unwrap_or("8").parse()?;
    let height = matches.value_of("height").unwrap_or("8").parse()?;
    let threshold = matches.value_of("threshold").unwrap_or("2").parse()?;

    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let paths: Vec<_> = reader.lines().flatten().collect();

    let mut images = vec![None; paths.len()];
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
            match (hash, modified) {
                (Some(hash), Some(modified)) => Some(ImageInfo {
                    path,
                    hash,
                    modified,
                }),
                _ => None,
            }
        })
        .collect_into_vec(&mut images);

    let images: Vec<_> = images.iter().flatten().collect();

    let mut similaritys: Vec<_> = images
        .iter()
        .enumerate()
        .map(|(i, image0)| {
            let mut similarity: Vec<_> = images[i..]
                .iter()
                .filter(|image1| distance(image0.hash, image1.hash, width * height) < threshold)
                .collect();
            similarity.sort_by_key(|k| Reverse(k.modified));
            similarity
        })
        .filter(|x| x.len() >= 2)
        .collect();

    similaritys.sort_by_key(|k| Reverse(k[0].modified));

    for s in similaritys.iter() {
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
