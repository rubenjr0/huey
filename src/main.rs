#![feature(trait_alias)]

use std::{fmt::Debug, str::FromStr};

use clap::{Parser, ValueEnum};
use eyre::{ContextCompat, Result};
use image::{io::Reader as ImageReader, DynamicImage, Rgb32FImage};
use itertools::Itertools;
use palette::{
    cast, color_difference::EuclideanDistance, rgb::Rgb, FromColor, IntoColor, Mix, Okhsl, Oklab,
    Saturate, Srgb,
};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

trait Color = Sync
    + Copy
    + Debug
    + FromColor<Rgb>
    + IntoColor<Rgb>
    + IntoColor<Okhsl>
    + Mix<Scalar = f32>
    + EuclideanDistance<Scalar = f32>;

#[derive(Clone, Debug, ValueEnum)]
enum InterpolationMode {
    Mix,
    Interpolate,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    image_path: String,
    palette_path: String,
    #[arg(short, long)]
    output_path: Option<String>,
    #[arg(value_enum, short = 'i', long)]
    interpolation_mode: Option<InterpolationMode>,
    #[arg(short = 'r', long = "rgb", help = "Uses OKLAB by default")]
    use_rgb: bool,
    #[arg(
        short = 'm',
        long,
        default_value_t = 1.0,
        help = "Replacement strength - 0: Don't replace, 1: Fully replace"
    )]
    mix_strength: f32,
    #[arg(short, long, help = "Apply saturation to the image")]
    saturation: Option<f32>,
}

fn load_palette<T>(path: &str) -> Result<Vec<T>>
where
    T: Color,
{
    let contents = std::fs::read_to_string(path)?;
    let colors = contents
        .trim()
        .replace("#", "")
        .split_whitespace()
        .unique()
        .map(|hex| {
            let rgb = Srgb::from_str(hex).expect("Invalid color representation");
            let rgb = rgb.into_format::<f32>();
            T::from_color(rgb)
        })
        .collect();
    Ok(colors)
}

fn closest_color<'a, T>(
    color: &Rgb,
    palette: &'a Vec<T>,
    interpolation_mode: &Option<InterpolationMode>,
    mix_strength: f32,
    saturation: Option<f32>,
) -> Option<Rgb>
where
    T: Color,
{
    let color = (*color).into_color();
    let mut palette: Vec<_> = palette
        .iter()
        .map(|c| (c, c.distance_squared(color)))
        .collect();
    palette.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal));
    let mut closest = palette.into_iter();
    let Some((c1, d1)) = closest.next() else {
        return None;
    };
    let new_color = if let Some(interpolation_mode) = interpolation_mode {
        let Some((c2, d2)) = closest.next() else {
            return None;
        };
        let factor = match interpolation_mode {
            InterpolationMode::Mix => 1.0 - d2 / (d1 + d2),
            InterpolationMode::Interpolate => 0.5,
        };
        c1.mix(*c2, factor)
    } else {
        *c1
    };
    let new_color = color.mix(new_color, mix_strength);
    let new_color: Okhsl = new_color.into_color();
    let new_color = if let Some(saturation) = saturation {
        new_color.saturate(saturation)
    } else {
        new_color
    };
    Some(new_color.into_color())
}

fn colorize<T>(
    image: &mut Rgb32FImage,
    palette_path: &str,
    interpolation_mode: Option<InterpolationMode>,
    mix_strength: f32,
    saturation: Option<f32>,
) -> Result<()>
where
    T: Color,
{
    let palette = load_palette(palette_path).expect("Could not open the palette file");
    let image_cast = cast::from_component_slice_mut::<Srgb<f32>>(image);
    image_cast
        .par_iter_mut()
        .try_for_each(|pixel| -> Result<()> {
            let closest = closest_color::<T>(
                pixel,
                &palette,
                &interpolation_mode,
                mix_strength,
                saturation,
            )
            .context("Not enough colors present in the palette")?;
            *pixel = Srgb::from_linear(closest.into_color());
            Ok(())
        })
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut image = ImageReader::open(args.image_path)?
        .with_guessed_format()?
        .decode()?
        .to_rgb32f();
    if args.use_rgb {
        colorize::<Rgb>(
            &mut image,
            &args.palette_path,
            args.interpolation_mode,
            args.mix_strength,
            args.saturation,
        )
    } else {
        colorize::<Oklab>(
            &mut image,
            &args.palette_path,
            args.interpolation_mode,
            args.mix_strength,
            args.saturation,
        )
    }?;

    let image = DynamicImage::try_from(image)?.to_rgb8();
    image.save(args.output_path.unwrap_or("colorized.png".to_string()))?;

    Ok(())
}
