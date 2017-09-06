fn uint32_from_bytes(bytes : [u8; 4]) -> u32 {
    return ((bytes[0] as u32) << 24) +
        ((bytes[1] as u32) << 16) +
        ((bytes[2] as u32) << 8) +
        bytes[3] as u32;
}

fn uint16_from_bytes(bytes : [u8; 2]) -> u16 {
    return ((bytes[0] as u16) << 8) + bytes[1] as u16;
}

pub struct FILMHeader {
    // Always 'FILM'
    #[allow(dead_code)]
    signature: String,
    pub length: usize,
    pub version: String,
    #[allow(dead_code)]
    unknown: Vec<u8>,
    pub fdsc: FDSC,
    pub stab: STAB,
}

impl FILMHeader {
    pub fn guess_length(data : &[u8]) -> usize {
        return uint32_from_bytes([data[4], data[5], data[6], data[7]]) as usize;
    }

    pub fn is_film_file(data : &[u8]) -> bool {
        let signature = String::from_utf8(data[0..4].to_vec()).unwrap();
        return signature == "FILM";
    }

    pub fn parse(data : &[u8]) -> Result<FILMHeader, &'static str> {
        let signature = String::from_utf8(data[0..4].to_vec()).unwrap();
        if signature != "FILM" {
            return Err("This is not a Sega FILM file!");
        }
        let length = uint32_from_bytes([data[4], data[5], data[6], data[7]]) as usize;

        return Ok(FILMHeader {
            signature: signature,
            length: length,
            version: String::from_utf8(data[8..12].to_vec()).unwrap(),
            unknown: data[12..16].to_vec(),
            fdsc: FDSC::parse(&data[16..48]),
            stab: STAB::parse(&data[48..length]),
        });
    }
}

pub struct FDSC {
    // Always 'FDSC'
    #[allow(dead_code)]
    signature: String,
    #[allow(dead_code)]
    length: u32,
    fourcc: String,
    pub height: u32,
    pub width: u32,
    // In practice always 24
    pub bpp: u8,
    pub channels: u8,
    // Always 8 or 32
    pub audio_resolution: u8,
    pub audio_compression: u8,
    pub audio_sampling_rate: u16,
}

impl FDSC {
    pub fn parse(data : &[u8]) -> FDSC {
        let signature_bytes = vec![
            data[0], data[1], data[2], data[3],
        ];
        let fourcc_bytes = vec![
            data[8], data[9], data[10], data[11],
        ];

        return FDSC {
            signature: String::from_utf8(signature_bytes).unwrap(),
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]),
            fourcc: String::from_utf8(fourcc_bytes).unwrap(),
            height: uint32_from_bytes([data[12], data[13], data[14], data[15]]),
            width: uint32_from_bytes([data[16], data[17], data[18], data[19]]),
            bpp: data[20],
            channels: data[21],
            audio_resolution: data[22],
            audio_compression: data[23],
            audio_sampling_rate: uint16_from_bytes([data[24], data[25]]),
        };
    }

    pub fn audio_codec(&self) -> &'static str {
        if self.audio_compression == 0 {
            return "pcm";
        } else {
            return "adx";
        }
    }

    pub fn human_readable_fourcc(&self) -> &'static str {
        if self.fourcc == "cvid" {
            return "Cinepak";
        } else {
            return "Raw video";
        }
    }
}

pub struct STAB {
    // Always 'STAB'
    #[allow(dead_code)]
    signature: String,
    #[allow(dead_code)]
    length: u32,
    // in Hz
    pub framerate: u32,
    // Number of entries in the sample table
    #[allow(dead_code)]
    entries: u32,
    pub sample_table: Vec<Sample>,
}

impl STAB {
    pub fn parse(data : &[u8]) -> STAB {
        let signature_bytes = vec![
            data[0], data[1], data[2], data[3],
        ];
        let entries = uint32_from_bytes([data[12], data[13], data[14], data[15]]);
        let mut samples = vec![];
        for i in 1..entries {
            let index = i as usize * 16;
            let sample = Sample::parse(&data[index..index + 16]);
            samples.push(sample);
        }

        return STAB {
            signature: String::from_utf8(signature_bytes).unwrap(),
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]),
            framerate: uint32_from_bytes([data[8], data[9], data[10], data[11]]),
            entries: entries,
            sample_table: samples,
        };
    }
}

pub struct Sample {
    pub offset: usize,
    pub length: usize,
    info1: [u8; 4],
    #[allow(dead_code)]
    info2: [u8; 4],
}

impl Sample {
    pub fn parse(data : &[u8]) -> Sample {
        return Sample {
            offset: uint32_from_bytes([data[0], data[1], data[2], data[3]]) as usize,
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]) as usize,
            info1: [data[8], data[9], data[10], data[11]],
            info2: [data[12], data[13], data[14], data[15]],
        }
    }

    // For the purpose of this program, we don't care about video data at all;
    // we just want to be able to identify which samples are audio.
    pub fn is_audio(&self) -> bool {
        return self.info1 == [255, 255, 255, 255];
    }
}
