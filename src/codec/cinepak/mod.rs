use utils::uint16_from_bytes;

// Referenced from https://multimedia.cx/mirror/cinepak.txt, and
// the FFmpeg source.

// According to FFmpeg, a couple of files have a different offset;
// it's detectable, so this could be improved to handle that.
const STRIP_START_OFFSET : usize = 12;

pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub strip_count: usize,
    pub keyframe: bool,
    pub strips: Vec<Strip>,
}

impl Frame {
    pub fn parse(data : &[u8]) -> Frame {
        let strip_count = uint16_from_bytes([data[8], data[9]]) as usize;

        let mut strips = vec![];
        // Strips can be relative in position to previous strips;
        // this value is kept throughout the loop to refer back to.
        let mut prev_y2 = 0;
        let mut current_offset : usize = 0;

        for i in 0..strip_count {
            let start_index = STRIP_START_OFFSET + current_offset;
            let strip_size = Strip::parse_strip_size(&data[start_index..start_index + 4]);

            let strip_data = &data[start_index..start_index + strip_size];
            strips.push(Strip::parse(strip_data, prev_y2));

            prev_y2 = strips[i].y2;
            current_offset += strip_size;
        }

        // TODO: This is also available at the sample level;
        //       is that value more accurate than this?
        let is_keyframe = strips.iter().any(|strip| strip.id == 0x10);

        return Frame {
            width: uint16_from_bytes([data[6], data[7]]) as usize,
            height: uint16_from_bytes([data[4], data[5]]) as usize,
            keyframe: is_keyframe,
            strip_count: strip_count,
            strips: strips,
        }
    }
}

pub struct Strip {
    pub id : u16,
    pub x1 : usize,
    pub x2 : usize,
    pub y1 : usize,
    pub y2 : usize,
    header : Vec<u8>,
    data   : Vec<u8>,
}

impl Strip {
    pub fn parse(data : &[u8], prev_y2 : usize) -> Strip {
        let y1;
        let y2;
        // 0 means relative to the previous strip
        if data[4] == 0 {
            y1 = prev_y2;
            y2 = y1 + uint16_from_bytes([data[8], data[9]]) as usize;
        } else {
            y1 = uint16_from_bytes([data[4], data[5]]) as usize;
            y2 = uint16_from_bytes([data[8], data[9]]) as usize;
        }

        let mut header = vec![];
        let mut strip_data = vec![];
        header.extend(data[0..12].iter().cloned());
        strip_data.extend(data[12..data.len()].iter().cloned());

        debug_assert!(header.len() == 12);
        debug_assert!((header.len() + strip_data.len()) == data.len());

        return Strip {
            id: uint16_from_bytes([data[0], data[1]]),
            x1: uint16_from_bytes([data[6], data[7]]) as usize,
            x2: uint16_from_bytes([data[10], data[11]]) as usize,
            y1: y1,
            y2: y2,
            header: header,
            data: strip_data,
        }
    }

    pub fn parse_strip_size(data : &[u8]) -> usize {
        // TODO: this might sometimes overshoot?
        return uint16_from_bytes([data[2], data[3]]) as usize;
    }
}

pub struct Vector {

}
