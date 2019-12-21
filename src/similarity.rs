use image::{self, DynamicImage, FilterType, GenericImageView};
use rayon::prelude::*;

trait ImageSimilar {
    fn similar(&self, other: &Self, width: u32, height: u32) -> u64;
}

impl ImageSimilar for DynamicImage {
    fn similar(&self, other: &DynamicImage, width: u32, height: u32) -> u64 {
        let mut hashs = vec![0; 2];
        [self, other]
            .par_iter()
            .map(|image| {
                image
                    .resize_exact(width, height, FilterType::Lanczos3)
                    .grayscale()
                    .calc_hash()
            })
            .collect_into_vec(&mut hashs);
        distance(hashs[0], hashs[1], width * height)
    }
}

pub trait CalcHash {
    fn calc_hash(&self) -> u64;
}

impl CalcHash for DynamicImage {
    fn calc_hash(&self) -> u64 {
        let sum_pixels: u64 = self.pixels().into_iter().map(|p| p.2[0] as u64).sum();
        let pixels: Vec<u64> = self.pixels().into_iter().map(|p| p.2[0] as u64).collect();
        let (width, height) = self.dimensions();
        let average = (sum_pixels as f64) / (width * height) as f64;
        pixels
            .iter()
            .fold((0, 1), |(hash, one), p| {
                if *p as f64 > average {
                    (hash | one, one << 1)
                } else {
                    (hash, one << 1)
                }
            })
            .0
    }
}

pub fn distance(hash1: u64, hash2: u64, pixels: u32) -> u64 {
    (0..pixels as u64)
        .map(|x| 1 << x)
        .filter(|&x| hash1 & x != hash2 & x)
        .count() as u64
}

