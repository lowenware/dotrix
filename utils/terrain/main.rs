use dotrix::terrain::{ColorMap, FalloffConfig, HeightMap, NoiseConfig};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    io::{self, Write},
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    Generate,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dotrix-terrain")]
pub struct Inputs {
    /// Path to .toml configuration file
    #[structopt(default_value = "configs/terrain.toml", short, long)]
    pub config: String,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    name: String,
    size: u32,
    heightmap: HeightmapSetup,
    falloff: Option<FalloffSetup>,
    moisture: Option<MoistureSetup>,
    colormap: Option<ColormapSetup>,
}

#[derive(Deserialize, Serialize)]
pub struct HeightmapSetup {
    file: String,
    falloff: bool,
    noise: NoiseConfig,
}

#[derive(Deserialize, Serialize)]
pub struct FalloffSetup {
    file: String,
    config: Option<FalloffConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct MoistureSetup {
    file: String,
    noise: NoiseConfig,
}

#[derive(Deserialize, Serialize)]
pub struct ColormapSetup {
    file: String,
    moisture: bool,
    colors: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let inputs = Inputs::from_args();
    let config: Config = toml::from_str(
        std::fs::read_to_string(std::path::Path::new(inputs.config.as_str()))?.as_str(),
    )?;

    // generate
    if let Some((file, falloff)) = config.falloff.as_ref().and_then(|setup| {
        setup
            .config
            .as_ref()
            .map(|falloff| (setup.file.as_ref(), falloff))
    }) {
        generate_falloff(file, config.size, falloff)?;
    }

    generate_heightmap(&config)?;

    generate_moisture(&config)?;

    generate_colormap(&config)?;

    Ok(())
}

fn generate_falloff(file: &str, size: u32, falloff: &FalloffConfig) -> Result<(), Box<dyn Error>> {
    print!("Generate falloff map... ");
    io::stdout().flush().ok();
    let heightmap = HeightMap::new_from_falloff("heightmap", size, falloff);
    let pixels = (0..size)
        .flat_map(|z| (0..size).map(move |x| (x, z)))
        .map(|(x, z)| (heightmap.value(x, z) * 255.0) as u8)
        .collect::<Vec<u8>>();

    image::GrayImage::from_raw(size, size, pixels)
        .expect("Could not generate heightmap pixels buffers")
        .save_with_format(&std::path::Path::new(file), image::ImageFormat::Png)?;
    println!("OK");
    Ok(())
}

fn generate_heightmap(config: &Config) -> Result<(), Box<dyn Error>> {
    print!("Generate heightmap... ");
    io::stdout().flush().ok();
    let mut heightmap =
        HeightMap::new_from_noise("heightmap", config.size, &config.heightmap.noise);
    if config.heightmap.falloff {
        if let Some(falloff_file) = config.falloff.as_ref().map(|setup| setup.file.as_str()) {
            let image = image::io::Reader::open(falloff_file)?.decode()?;
            let size = image.width();
            let bytes = image.into_bytes();
            let falloff_heightmap = HeightMap::new_from_bytes("falloff", size, &bytes);
            heightmap.subtract(&falloff_heightmap);
        }
    }
    heightmap.write_to_file(
        std::path::Path::new(config.heightmap.file.as_str()),
        image::ImageFormat::Png,
    )?;
    println!("OK");
    Ok(())
}

fn generate_moisture(config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(moisture) = config.moisture.as_ref() {
        print!("Generate moisture map... ");
        io::stdout().flush().ok();
        let heightmap = HeightMap::new_from_noise("moisturemap", config.size, &moisture.noise);
        heightmap.write_to_file(
            std::path::Path::new(moisture.file.as_str()),
            image::ImageFormat::Png,
        )?;
        println!("OK");
    }
    Ok(())
}

fn generate_colormap(config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(colormap_config) = config.colormap.as_ref() {
        print!("Generate colormap... ");
        io::stdout().flush().ok();
        let moisture = if colormap_config.moisture {
            config.moisture.as_ref().map(|moisture| {
                let image = image::io::Reader::open(moisture.file.as_str())
                    .expect("Could not open moisture map")
                    .decode()
                    .expect("Could not decode moisture map");
                let size = image.width();
                let bytes = image.into_bytes();
                HeightMap::new_from_bytes("moisturemap", size, &bytes)
            })
        } else {
            None
        };
        let image = image::io::Reader::open(config.heightmap.file.as_str())
            .expect("Could not open heightmap file")
            .decode()
            .expect("Could not decode heightmap map");
        let size = image.width();
        let bytes = image.into_bytes();
        let heightmap = HeightMap::new_from_bytes("heightmap", size, &bytes);

        let colors = colormap_config
            .colors
            .iter()
            .map(|hex| {
                [
                    u8::from_str_radix(&hex[0..2], 16)
                        .expect("Red channel in color must be a valid HEX number"),
                    u8::from_str_radix(&hex[2..4], 16)
                        .expect("Green channel in color must be a valid HEX number"),
                    u8::from_str_radix(&hex[4..6], 16)
                        .expect("Blue channel in color must be a valid HEX number"),
                ]
            })
            .collect::<Vec<_>>();

        let colormap = ColorMap::new("colormap", colors, 0.2);

        let pixels = (0..size)
            .flat_map(|x| (0..size).map(move |z| (x, z)))
            .flat_map(|(x, z)| {
                let moisture = moisture
                    .as_ref()
                    .map(|moisture| moisture.value(x, z))
                    .unwrap_or(0.0);
                colormap.color(heightmap.value(x, z), moisture).into_iter()
            })
            .collect::<Vec<u8>>();

        image::RgbImage::from_raw(size, size, pixels)
            .expect("Could not generate colormap pixels buffers")
            .save_with_format(
                &std::path::Path::new(colormap_config.file.as_str()),
                image::ImageFormat::Png,
            )?;
        println!("OK");
    }
    Ok(())
}
