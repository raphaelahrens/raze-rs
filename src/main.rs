use std::time::Duration;
use eyre::{eyre, Result};
use clap::{Parser, Subcommand, ValueEnum};

use razer_rs::{pack, Off, Breath, Spectrum, SCROLLWHEEL, LOGO, Colour, Static};


const VENDOR_ID:u16 = 0x1532;
const PRODUCT_ID:u16 = 0x005c;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The LED to be controlled
    #[arg(value_enum)]
    led: Led,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Led {
    /// Run swiftly
    Logo,
    /// Crawl slowly but steadily
    ///
    /// This paragraph is ignored because there is no long help text for possible values.
    Scrollwheel,
}

impl From<Led> for u8 {
    fn from(led: Led) -> Self {
        match led {
            Led::Logo=> LOGO,
            Led::Scrollwheel => SCROLLWHEEL,
        }

    }
}
#[derive(Debug, Subcommand)]
enum Command {
    /// Switch LED off
    Off,
    /// Let the LED show the whole spectrum of colours
    Spectrum,
    /// Let the LED breath
    Breath {
        colour: Option<Colour>
    },
    /// let the LED  to a static colour
    Static{
        colour: Colour
    },
}

fn main() -> Result<()>{
    let vendor_id = VENDOR_ID;
    let product_id = PRODUCT_ID;

    let cli = Cli::parse();

    let led = cli.led.into();

    dbg!(&cli);

    let msg:Vec<u8> = match cli.command {
        Command::Off => pack(Off::new(led))?,
        Command::Spectrum => pack(Spectrum::new(led))?,
        Command::Breath{colour} =>{
            pack(Breath::new(led, colour))?
        },
        Command::Static{colour} => pack(Static::new(led, colour))?,
    };

    let context = libusb::Context::new()?;
    
    let razer_handle = match context.open_device_with_vid_pid(vendor_id, product_id) {
       None => {
           return Err(eyre!("Could not find Device with VID: {} and PID: {}", vendor_id, product_id));
       },
       Some(razer_handle) => razer_handle,
    };
    

    razer_handle.write_control(0x21, 0x09, 0x300, 0x01, &msg, Duration::new(1,0))?;
    Ok(())
}
