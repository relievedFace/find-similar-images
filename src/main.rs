use clap::{App, Arg};
use find_similar_images::image_info::ImageInfo;
use find_similar_images::similarity::distance;
use find_similar_images::similarity::CalcHash;
use image;
use image::FilterType;
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::BTreeSet;
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

    let similaritys: Vec<_> = images
        .iter()
        .combinations(2)
        .filter(|image| distance(image[0].hash, image[1].hash, width * height) < threshold)
        .collect();

    let mut similarity_images_list = vec![];

    let mut check = vec![true; similaritys.len()];
    for (i, edge0) in similaritys.iter().enumerate() {
        if check[i] {
            check[i] = false;

            let mut set = BTreeSet::new();
            set.insert(edge0[0]);
            set.insert(edge0[1]);

            for (j, edge1) in similaritys.iter().enumerate() {
                if i != j && check[j] && (set.contains(&edge1[0]) || set.contains(&edge1[1])) {
                    check[j] = false;

                    set.insert(edge1[0]);
                    set.insert(edge1[1]);
                }
            }

            let mut similarity_images: Vec<_> = set.iter().cloned().collect();
            similarity_images.sort_by_key(|k| Reverse(k.modified));
            similarity_images_list.push(similarity_images);
        }
    }

    similarity_images_list.sort_by_key(|k| Reverse(k[0].modified));

    for s in similarity_images_list.iter() {
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
