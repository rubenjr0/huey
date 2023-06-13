use std::{fmt::Debug, str::FromStr};

use clap::{Parser, ValueEnum};
use eyre::{ContextCompat, Result};
use image::{io::Reader as ImageReader, DynamicImage, Rgb32FImage};
use palette::{
    cast, color_difference::EuclideanDistance, rgb::Rgb, FromColor, IntoColor, Mix, Oklab, Srgb,
};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

#[derive(Clone, Debug, ValueEnum)]
enum InterpolationMode {
    Mix,
    Interpolate,
}

fn load_palette<T>(path: &str) -> Result<Vec<T>>
where
    T: FromColor<Rgb>,
{
    let contents = std::fs::read_to_string(path)?;
    let colors: Vec<_> = contents
        .trim()
        .split_whitespace()
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
) -> Option<Rgb>
where
    T: Copy
        + Debug
        + Mix<Scalar = f32>
        + FromColor<Rgb>
        + IntoColor<Rgb>
        + EuclideanDistance<Scalar = f32>,
{
    let color: T = (*color).into_color();
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
    Some(new_color.into_color())
}

fn colorize<T>(
    image: &mut Rgb32FImage,
    palette_path: &str,
    interpolation_mode: Option<InterpolationMode>,
) -> Result<()>
where
    T: Sync
        + Mix<Scalar = f32>
        + Copy
        + Debug
        + FromColor<Rgb>
        + IntoColor<Rgb>
        + EuclideanDistance<Scalar = f32>,
{
    let palette = load_palette(palette_path).expect("Could not open the palette file");
    let image_cast = cast::from_component_slice_mut::<Srgb<f32>>(image);
    image_cast
        .par_iter_mut()
        .try_for_each(|pixel| -> Result<()> {
            let closest = closest_color::<T>(pixel, &palette, &interpolation_mode)
                .context("Not enough colors present in the palette")?;
            *pixel = Srgb::from_linear(closest.into_color());
            Ok(())
        })
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
    #[arg(short = 'r', long, help = "Uses OKLAB by default")]
    use_rgb: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut image = ImageReader::open(args.image_path)?
        .with_guessed_format()?
        .decode()?
        .to_rgb32f();
    if args.use_rgb {
        colorize::<Rgb>(&mut image, &args.palette_path, args.interpolation_mode)
    } else {
        colorize::<Oklab>(&mut image, &args.palette_path, args.interpolation_mode)
    }?;

    let image = DynamicImage::from(image).to_rgb8();
    image.save(args.output_path.unwrap_or("colorized.png".to_string()))?;

    Ok(())
}
