use serde::{Serialize};
use std::str::FromStr;
use bincode::Options; // TODO: Replace with proper serializer 
use std::convert::TryFrom;

// MESSAGE values
// const NOSTORE:u8 = 0x0;
const VARSTORE:u8 = 0x1;

const MAX_LEN:usize = 90;

// LEDS 
pub const SCROLLWHEEL:u8 = 0x01;
pub const LOGO:u8 = 0x04;

#[derive(Serialize, Debug)]
struct Header {
     status:u8,
     transaction_id:u8,
     remaining_packets:u16,
     protocol_type:u8,
     args_len:u8,
     command_class:u8,
     command_id:u8,
}

impl Header {
    const fn new(args_len:u8, command_class:u8, command_id:u8) -> Self {
        Self{
             status: 0x00,
             transaction_id: 0x3F,
             remaining_packets:0x0000,
             protocol_type:0x00,
             args_len,
             command_class,
             command_id,
        }
    }
}
#[derive(Clone, Serialize, Debug)]
pub struct Colour {
    r:u8,
    g:u8,
    b:u8,
}

impl TryFrom<&str> for Colour {
    type Error = std::num::ParseIntError;
    fn try_from(hex_str: &str) -> Result<Self, Self::Error> {
        // u8::from_str_radix(src: &str, radix: u32) converts a string
        // slice in a given base to u8
        let r: u8 = u8::from_str_radix(&hex_str[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex_str[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex_str[5..7], 16)?;

        Ok(Self{ r, g, b })
    }
}

impl FromStr for Colour {
    type Err = std::num::ParseIntError;

    // Parses a color hex code of the form '#rRgGbB..' into an
    // instance of 'RGB'
    fn from_str(hex_code: &str) -> Result<Self, Self::Err> {
    
        // u8::from_str_radix(src: &str, radix: u32) converts a string
        // slice in a given base to u8
        let r: u8 = u8::from_str_radix(&hex_code[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex_code[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex_code[5..7], 16)?;

        Ok(Self{ r, g, b })
    }
}

#[derive(Serialize, Debug)]
struct BaseArgs {
    set: u8,
    led: u8,
    effect_id:u8,
    arg1:u8,
    arg2:u8,
    arg3:u8,
}

impl BaseArgs {
    const fn new(led:u8, effect_id:u8, arg1:u8, arg2:u8, arg3:u8) -> Self {
        Self{
            set: VARSTORE,
            led,
            effect_id,
            arg1,
            arg2,
            arg3,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Off {
    header: Header,
    base_args: BaseArgs,
}
impl Off {
    #[must_use]
    pub const fn new(led:u8) -> Self {
        let args_len:u8 = 6;
        let effect_id:u8 = 0x00;
        Self{
            header: Header::new(args_len, 0x0F, 0x02),
            base_args: BaseArgs::new(led, effect_id, 0x0, 0x0, 0x01),
        }
    }
}


#[derive(Serialize, Debug)]
pub struct Static {
    header: Header,
    base_args: BaseArgs,
    colour:Colour,

}
impl Static {
    #[must_use]
    pub const fn new(led:u8, colour:Colour) -> Self {
        let args_len:u8 = 9;
        let effect_id:u8 = 0x01;
        Self{
            header: Header::new(args_len, 0x0F, 0x02),
            base_args: BaseArgs::new(led, effect_id, 0x0, 0x0, 0x01),
            colour,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Breath {
    header: Header,
    base_args: BaseArgs,
    colour:Colour,

}
impl Breath {
    #[must_use]
    pub const fn new(led:u8, colour: Option<Colour>) -> Self {
        let effect_id:u8 = 0x02;
        
        let (colour, base_args, args_len) = match colour {
            None => {
                (
                    Colour{r:0,g:0,b:0},
                    BaseArgs::new(led, effect_id, 0x0, 0x0, 0x00),
                    6,
                    )
            },
            Some(colour) => {
                (
                    colour,
                    BaseArgs::new(led, effect_id, 0x1, 0x0, 0x01),
                    9,
                    )
            }
        };
    
        Self{
            header: Header::new(args_len, 0x0F, 0x02),
            base_args,
            colour,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Spectrum {
    header: Header,
    base_args: BaseArgs,

}
impl Spectrum {
    #[must_use]
    pub const fn new(led:u8) -> Self {
        let args_len:u8 = 6;
        let effect_id:u8 = 0x03;
        Self{
            header: Header::new(args_len, 0x0F, 0x02),
            base_args: BaseArgs::new(led, effect_id, 0x0, 0x0, 0x00),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Brightness {
    header: Header,
    set: u8,
    led: u8,

}
impl Brightness {
    #[must_use]
    pub const fn new(led:u8) -> Self {
        let args_len:u8 = 3;
        Self{
            header: Header::new(args_len, 0x0F, 0x04),
            set: VARSTORE,
            led,

        }
    }
}

fn checksum(bytes:&[u8]) -> u8 {
    bytes.iter().skip(2).fold(0, |acc, x| acc ^ x)
}

pub fn pack<T:Serialize>(msg:T) -> Result<Vec<u8>, bincode::Error> {
    let s = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_big_endian();
    let mut bytes = s.serialize(&msg)?;
    
    let checksum = checksum(&bytes);

    bytes.resize(MAX_LEN, 0);

    bytes[MAX_LEN -2] = checksum;

    Ok(bytes)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn off() {
        let ref_msg:Vec<u8> = vec![0x00, 0x3f, 0x00, 0x00, 0x00, 0x06, 0x0f, 0x02,
                                   0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x0a, 0x00,];

        let msg =  Off::new(SCROLLWHEEL);
        assert_eq!(ref_msg, pack(msg).unwrap());
    }
}
