use std::{ffi::OsString, num::NonZeroU32, path::Path};

use clap::Parser;

use anyhow::{anyhow, Context, Result};
use image::RgbaImage;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Parser, Debug)]
enum Action {
    /// Converts an aseprite file containing a single frame to a png file
    Convert(Convert),
    Assemble(Assemble),
    Separate(Separate),
}

#[derive(Parser, Debug)]
struct Convert {
    input_file: OsString,
    output_file: OsString,
}

#[derive(Parser, Debug)]
struct Assemble {
    input_file: OsString,
    output_file: OsString,
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    tags: Vec<String>,
    #[arg(short, long)]
    number_of_frames_from_each: Option<NonZeroU32>,
    #[arg(short, long)]
    columns: Option<NonZeroU32>,
}

#[derive(Parser, Debug)]
struct Separate {
    input_file: OsString,
    output_directory: OsString,
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    tags: Vec<String>,
}

impl Convert {
    fn convert(&self) -> Result<()> {
        let input_path = Path::new(&self.input_file);
        let input_file = asefile::AsepriteFile::read_file(input_path)
            .with_context(|| format!("{} can't be loaded", input_path.display()))?;

        if input_file.num_frames() != 1 {
            return Err(anyhow!(
                "Convert only supports a single frame, {} frames found in {}",
                input_file.num_frames(),
                input_path.display()
            ));
        }

        let input_image = input_file.frame(0).image();

        let output_path = Path::new(&self.output_file);
        input_image
            .save(output_path)
            .with_context(|| format!("Cannot save image to {}", output_path.display()))?;

        Ok(())
    }
}

impl Assemble {
    fn assemble(&self) -> Result<()> {
        let input_path = Path::new(&self.input_file);
        let input_file = asefile::AsepriteFile::read_file(input_path)
            .with_context(|| format!("{} can't be loaded", input_path.display()))?;

        let single_width = input_file.width();
        let single_height = input_file.height();

        let number_of_frames_from_each = self
            .number_of_frames_from_each
            .map(|x| x.get())
            .unwrap_or(1) as usize;

        let total_image_count = self.tags.len() * number_of_frames_from_each;

        let total_columns = self
            .columns
            .map(|x| x.get() as usize)
            .unwrap_or(total_image_count);

        let total_width = total_columns * single_width;

        let total_height = (total_image_count.div_ceil(total_columns)) * single_height;

        let mut output_image = RgbaImage::new(total_width as u32, total_height as u32);

        for (tag_count, tag) in self.tags.iter().enumerate() {
            let image_tag = input_file.tag_by_name(tag).with_context(|| {
                format!("{tag} doesn't exist in image {}", input_path.display())
            })?;

            if (image_tag.to_frame() as i32 - image_tag.from_frame() as i32 + 1)
                < (number_of_frames_from_each as i32)
            {
                return Err(anyhow!(
                    "Tag {tag} in file {} doesn't contain enough frames, it has {} but we need {number_of_frames_from_each}",
                    input_path.display(),
                    image_tag.to_frame() as i32- image_tag.from_frame() as i32,
                ));
            }

            for i in 0..number_of_frames_from_each {
                let image_count = tag_count * number_of_frames_from_each + i;
                let x_image = image_count % total_columns;
                let y_image = image_count / total_columns;
                let x_pixel = x_image * single_width;
                let y_pixel = y_image * single_height;

                let frame = input_file.frame(image_tag.from_frame() + i as u32);
                image::imageops::replace(
                    &mut output_image,
                    &frame.image(),
                    x_pixel as u32,
                    y_pixel as u32,
                );
            }
        }

        let output_path = Path::new(&self.output_file);
        output_image
            .save(output_path)
            .with_context(|| format!("Cannot save image to {}", output_path.display()))?;

        Ok(())
    }
}

impl Separate {
    fn separate(&self) -> Result<()> {
        let input_path = Path::new(&self.input_file);
        let input_file = asefile::AsepriteFile::read_file(input_path)
            .with_context(|| format!("{} can't be loaded", input_path.display()))?;

        let output_path = Path::new(&self.output_directory);

        for tag in self.tags.iter() {
            let image_tag = input_file.tag_by_name(tag).with_context(|| {
                format!("{tag} doesn't exist in image {}", input_path.display())
            })?;

            let frame = input_file.frame(image_tag.from_frame());

            let image_output_path = output_path.join(format!("{tag}.png"));
            frame
                .image()
                .save(&image_output_path)
                .with_context(|| format!("Cannot save image to {}", image_output_path.display()))?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Args::parse();

    match cli.action {
        Action::Convert(convert) => convert.convert()?,
        Action::Assemble(assemble) => assemble.assemble()?,
        Action::Separate(separate) => separate.separate()?,
    }

    Ok(())
}
