use std::time::Duration;
use clap::{Arg, App, SubCommand};
use std::str::FromStr;

use razer_rs::{pack, Off, Breath, Spectrum, SCROLLWHEEL, LOGO, Colour, Static};


const VENDOR_ID:u16 = 0x1532;
const PRODUCT_ID:u16 = 0x005c;

// LEDs commands
const LOGO_STR:&'static str = "logo";
const SCROLLWHEEL_STR:&'static str = "scrollwheel";

// EFFECT commands
const OFF:&str = "off";
const BREATH:&str = "breath";
const SPECTRUM:&str = "spectrum";
const STATIC:&str = "static";

fn main() {
    let matches = App::new("Razers led")
                          .version("0.1.0")
                          .author("Raphael Ahrens <raphaelahrens@googlemail.com>")
                          .about("Set the effect and sometimes Color of the Mouse LEDs")
                          .arg(Arg::with_name("LED")
                               .help("The LED to be controlled")
                               .possible_values(&[LOGO_STR, SCROLLWHEEL_STR])
                               .required(true)
                               )
                          .subcommand(SubCommand::with_name(OFF)
                                      .about("Switch LED off"))
                          .subcommand(SubCommand::with_name(SPECTRUM)
                                      .about("Let the LED show the whole spectrum"))
                          .subcommand(SubCommand::with_name(BREATH)
                                      .about("Let the LED breath")
                                      .arg(Arg::with_name("COLOUR")
                                           .help("Color which the LED shall glow in.")
                                           .required(false)
                                           )
                                      )
                          .subcommand(SubCommand::with_name(STATIC)
                                      .about("Set the LED to a static color")
                                      .arg(Arg::with_name("COLOUR")
                                           .help("Color which the LED shall glow in.")
                                           .required(true)
                                           )
                                       )
                          .get_matches();

    let vendor_id = VENDOR_ID;
    let product_id = PRODUCT_ID;

    let led = match matches.value_of("LED").unwrap(){
        LOGO_STR => LOGO,
        SCROLLWHEEL_STR => SCROLLWHEEL,
        _ => {
           panic!("Unknown LED args");
        }
    };

    let msg:Vec<u8> = match matches.subcommand() {
        (OFF,  Some(_sub_m)) => pack(Off::new(led)),
        (SPECTRUM, Some(_sub_m)) => pack(Spectrum::new(led)),
        (BREATH,   Some(sub_m)) => {
            match sub_m.value_of("COLOUR") {
                Some(x) => match Colour::from_str(x) {
                    Ok(colour) => pack(Breath::new_static_colour(led, colour)),
                    Err(_) => panic!("")
                },
                None => pack(Breath::new(led))
            }
        },
        (STATIC, Some(sub_m)) => {
            match Colour::from_str(sub_m.value_of("COLOUR").unwrap()) {
                Ok(colour) => pack(Static::new(led, colour)),
                Err(_) => panic!("")
            }
        },
        _                 => {panic!("");},
    };

    let context = libusb::Context::new().unwrap();
    
    let razer_handle = match context.open_device_with_vid_pid(vendor_id, product_id) {
       None => {
           panic!("Could not find Device with VID: {} and PID: {}", vendor_id, product_id);
       },
       Some(razer_handle) => razer_handle,
    };
    

    let result = razer_handle.write_control(0x21, 0x09, 0x300, 0x01, &msg, Duration::new(1,0));
    
    if let Err(e) = result {
        panic!("Error {}", e);
    }
}
