use std::{ffi::OsString, path::Path};

use clap::Parser;

use anyhow::{anyhow, Context, Result};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Parser, Debug)]
enum Action {
    /// Converts an aseprite file containing a single frame to a png file
    Convert(Convert),
}

#[derive(Parser, Debug)]
struct Convert {
    input_file: OsString,
    output_file: OsString,
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

fn main() -> Result<()> {
    let cli = Args::parse();

    match cli.action {
        Action::Convert(convert) => convert.convert()?,
    }

    Ok(())
}
