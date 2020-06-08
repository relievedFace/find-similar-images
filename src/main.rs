use clap::{App, Arg};
use find_similar_images::image_info::ImageInfo;
use find_similar_images::similarity::distance;
use find_similar_images::similarity::CalcHash;
use image;
use image::FilterType;
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};
use std::sync::Mutex;
use std::time::SystemTime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("find  images")
        .version("0.1.0")
        .author("relievedFace <relievedFace@gmaile.com>")
        .about("find similarity images")
        .arg(
            Arg::with_name("width")
                .help("default value: 8")
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
        )
        .arg(
            Arg::with_name("input_cache_file")
                .short("i")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output_cache_file")
                .short("o")
                .takes_value(true),
        );

    let matches = app.get_matches();
    let width = matches.value_of("width").unwrap_or("8").parse()?;
    let height = matches.value_of("height").unwrap_or("8").parse()?;
    let threshold = matches.value_of("threshold").unwrap_or("2").parse()?;
    let input_catch_file_path = matches.value_of("input_cache_file");
    let output_catch_file_path = matches.value_of("output_cache_file");

    let cache = read_cache(&input_catch_file_path.map(|x| x.to_string()));
    let cache = Mutex::new(&cache);

    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());

    let paths: Vec<_> = reader.lines().flatten().collect();

    let images: Vec<_> = paths
        .into_par_iter()
        .map(|path| {
            let path = path.replace(r" ", r"\ ");

            let metadata = fs::metadata(&path).ok()?;
            let modified = metadata
                .modified()
                .ok()?
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?
                .as_secs();
            let file_size = metadata.len();

            let cache_image = {
                let cache = *cache.lock().ok()?;
                cache.as_ref().map(|cache| cache.get(&path)).flatten()
            };

            let hash = match cache_image {
                Some(image) if image.modified == modified && image.file_size == file_size => {
                    image.hash
                }
                _ => image::open(&path)
                    .map(|image| {
                        image
                            .resize_exact(width, height, FilterType::Lanczos3)
                            .grayscale()
                            .calc_hash()
                    })
                    .ok()?,
            };
            Some(ImageInfo {
                path,
                hash,
                file_size,
                modified,
            })
        })
        .collect();

    let images: Vec<_> = images.iter().flatten().cloned().collect();

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

    write_result(&similarity_images_list)?;

    if let Some(path) = output_catch_file_path {
        write_cache(path, &images)?;
    }

    Ok(())
}

fn read_cache(path: &Option<String>) -> Option<HashMap<String, ImageInfo>> {
    path.as_ref().map(|path| {
        let cache_file =
            fs::File::open(&path).expect(&format!("faile to open input cache file: {}", path));
        let cache_file_buffer = BufReader::new(cache_file);
        let mut reader = csv::Reader::from_reader(cache_file_buffer);
        let mut cache = HashMap::new();

        for (i, result) in reader.deserialize().enumerate() {
            let image: ImageInfo =
                result.expect(&format!("parse error file {}, line {}", path, i + 1));
            cache.insert(image.path.clone(), image);
        }
        cache
    })
}

fn write_cache(path: &str, images: &Vec<ImageInfo>) -> Result<(), Box<dyn std::error::Error>> {
    let cache_file =
        fs::File::create(&path).expect(&format!("faile to open input cache file: {}", path));
    let mut cache_file_buffer = BufWriter::new(cache_file);
    let mut writer = csv::Writer::from_writer(vec![]);

    for image in images {
        writer.serialize(&image)?;
    }
    let data = String::from_utf8(writer.into_inner()?)?;

    dbg!(&data);
    write!(cache_file_buffer, "{}", data)?;
    Ok(())
}

fn write_result(
    similarity_images_list: &Vec<Vec<&ImageInfo>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    for s in similarity_images_list.iter() {
        writeln!(
            writer,
            "{}",
            s.iter()
                .map(|x| x.path.to_string().replace(r" ", r"\ "))
                .collect::<Vec<_>>()
                .join(" "),
        )?;
    }
    Ok(())
}
