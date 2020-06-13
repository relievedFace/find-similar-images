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
    let input_cache_file_path = matches.value_of("input_cache_file");
    let output_cache_file_path = matches.value_of("output_cache_file");

    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());

    let paths: Vec<_> = reader.lines().flatten().collect();
    let cache = input_cache_file_path
        .map(|path| read_cache(path).ok())
        .flatten();
    let (mut cache_images, nocache_image_paths) = if let Some(cache) = cache {
        split_cached_and_nocached(&paths, &cache)?
    } else {
        (vec![], paths)
    };

    let images: Vec<_> = nocache_image_paths
        .into_par_iter()
        .map(|path| {
            let modified = file_modified_as_sec(&path).ok()?;
            let file_size = file_size(&path).ok()?;
            let hash = image::open(&path)
                .map(|image| {
                    image
                        .resize_exact(width, height, FilterType::Lanczos3)
                        .grayscale()
                        .calc_hash()
                })
                .ok()?;

            Some(ImageInfo {
                path,
                hash,
                file_size,
                modified,
            })
        })
        .collect();

    let mut images: Vec<_> = images.iter().flatten().cloned().collect();
    images.append(&mut cache_images);

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

    if let Some(path) = output_cache_file_path {
        write_cache(path, &images)?;
    }

    Ok(())
}

fn read_cache(path: &str) -> Result<HashMap<String, ImageInfo>, Box<dyn std::error::Error>> {
    let cache_file = fs::File::open(path).expect(&format!("Faile to open file: {}", path));
    let cache_file_buffer = BufReader::new(cache_file);
    let mut reader = csv::Reader::from_reader(cache_file_buffer);
    let mut cache = HashMap::new();

    for (i, result) in reader.deserialize().enumerate() {
        let image: ImageInfo = result.expect(&format!("Error! file: {}, line: {}", path, i + 1));
        cache.insert(image.path.clone(), image);
    }

    Ok(cache)
}

fn write_cache(path: &str, images: &Vec<ImageInfo>) -> Result<(), Box<dyn std::error::Error>> {
    let cache_file = fs::File::create(path).expect(&format!("Faile to open file: {}", path));
    let mut cache_file_buffer = BufWriter::new(cache_file);
    let mut writer = csv::Writer::from_writer(vec![]);

    for image in images {
        writer.serialize(&image)?;
    }

    let data = String::from_utf8(writer.into_inner()?)?;

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
                .map(|x| format!(r#""{}""#, x.path.replace(r#"""#, r#"\""#)).to_string())
                .collect::<Vec<_>>()
                .join(" "),
        )?;
    }

    Ok(())
}

fn split_cached_and_nocached(
    paths: &Vec<String>,
    cache: &HashMap<String, ImageInfo>,
) -> Result<(Vec<ImageInfo>, Vec<String>), Box<dyn std::error::Error>> {
    let mut cache_images = vec![];
    let mut nocache_image_paths = vec![];
    for path in paths {
        let modified = file_modified_as_sec(&path)?;
        let file_size = file_size(&path)?;
        match cache.get(path) {
            Some(image) if image.modified == modified && image.file_size == file_size => {
                cache_images.push(image.clone())
            }
            _ => nocache_image_paths.push(path.clone()),
        }
    }
    Ok((cache_images, nocache_image_paths))
}

fn file_modified_as_sec(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(&path)?;
    let modified = metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(modified)
}

fn file_size(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(&path)?;
    Ok(metadata.len())
}
