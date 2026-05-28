use crate::bus::bus::Bus;
use crate::utils::test_bit;

const SPRITE_ATTRIBUTE_TABLE_START: u16 = 0xFE00;
const SPRITE_ATTRIBUTE_TABLE_END: u16 = 0xFE9F;
pub const NUM_SPRITES: usize = 40;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    y_position: u8,     // byte 0
    x_position: u8,     // 1
    pub tile_index: u8, // 2
    pub attributes: Attributes,
}

#[derive(Clone, Copy, Debug)]
pub struct Attributes {
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    pub dmg_palette: bool, // if this bit is 0, palette is from 0xFF48 otherwise 0xFF49
}

#[derive(num_enum::IntoPrimitive)]
#[repr(u8)]
enum AttributeBits {
    Priority = 7,
    YFlip = 6,
    XFlip = 5,
    DmgPalette = 4,
    // 3-0 are only used by CGB
}

impl Sprite {
    pub fn get_x(&self) -> u8 {
        self.x_position
    }

    pub fn get_y(&self) -> u8 {
        self.y_position
    }
}

pub fn read_sprite_attribute_table(bus: &Bus) -> [Option<Sprite>; NUM_SPRITES] {
    let mut output: [Option<Sprite>; NUM_SPRITES] = [None; NUM_SPRITES];

    let mut index = 0;
    for i in (SPRITE_ATTRIBUTE_TABLE_START..SPRITE_ATTRIBUTE_TABLE_END).step_by(4) {
        let sprite = Sprite {
            y_position: bus.read_byte(i),
            x_position: bus.read_byte(i + 1),
            tile_index: bus.read_byte(i + 2),
            attributes: Attributes::new(bus.read_byte(i + 3)),
        };

        output[index] = Some(sprite);
        index += 1;
    }

    return output;
}

impl Attributes {
    pub fn new(byte: u8) -> Self {
        Attributes {
            priority: test_bit(byte, AttributeBits::Priority.into()),
            y_flip: test_bit(byte, AttributeBits::YFlip.into()),
            x_flip: test_bit(byte, AttributeBits::XFlip.into()),
            dmg_palette: test_bit(byte, AttributeBits::DmgPalette.into()),
        }
    }
}
