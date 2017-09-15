//! This module contains types for parsing the header of a Sega FILM file.
//! The FILM container header has some basic metadata, and encapsulates two additional chunks:
//!
//! * FDSC, the format descriptor, which contains information about the container's contents
//! * STAB, the sample table, which contains information about each sample within the container

use utils::{uint16_from_bytes, uint32_from_bytes};

pub enum AudioCodec {
    PCM,
    ADX,
    Unknown,
}

impl AudioCodec {
    pub fn name(&self) -> &'static str {
        use self::AudioCodec::*;

        match *self {
            PCM => "pcm",
            ADX => "adx",
            Unknown => "unknown"
        }
    }
}

/// Represents the header of a FILM container.
/// Parsing a FILMHeader provides all of the information necessary to read
/// the file, including the data from the sub-chunks.
/// Most of the time, you can safely work from parsing the FILMHeader at the
/// start of a file without needing to manually parse the FDSC or STAB segments.
///
/// Since the FILM header is variable size, a method is provided to help you
/// calculate how many bytes you'll need to read in order to parse the header.
/// For example:
///
/// ```
/// let file = File::open("myfile.cpk")?;
/// let mut buf = vec![];
/// // Start with only 8 bytes, so we don't waste memory
/// file.take(8).read(&mut buf)?;
///
/// // Check if it's a FILM file before going any further!
/// if !FILMHeader::is_film_file(&buf) {
///     println!("Oh no!");
///     exit(1);
/// }
///
/// // Figure out how large the header is, and read the rest of its contents
/// let bytes_to_read = FILMHeader::guess_length(&buf);
/// file.take(bytes_to_read - 8).read(&mut buf)?;
///
/// // Now we're ready to parse!
/// let header = FILMHeader::parse(&buf)?;
/// ```
pub struct FILMHeader {
    // Always 'FILM'
    #[allow(dead_code)]
    signature: String,
    /// The size of the header, in bytes.
    pub length: usize,
    /// The version of the FILM file. This is mostly useful for predicting idiosyncrasies
    /// or predicting what the stream formats will be.
    pub version: String,
    #[allow(dead_code)]
    unknown: Vec<u8>,
    /// The parsed FDSC data.
    pub fdsc: FDSC,
    /// The parsed STAB data.
    pub stab: STAB,
}

impl FILMHeader {
    /// Guesses the length of the FILM header based on the supplied data.
    /// This is useful for determining how many bytes to pass when parsing a
    /// FILM file.
    ///
    /// `data` is a slice which is assumed to contain the beginning portion of
    /// a FILM file; it must contain at least the first 8 bytes of data.
    /// This doesn't guarantee that the passed data actually represents a FILM file;
    /// if it doesn't, the guess will not be meaningful.
    pub fn guess_length(data : &[u8]) -> usize {
        return uint32_from_bytes([data[4], data[5], data[6], data[7]]) as usize;
    }

    /// Determines whether the passed data comes from a FILM file.
    /// `data` is a slice which is assumed to contain the beginning portion of
    /// a FILM file; it must contain at least the first 4 bytes of data.
    pub fn is_film_file(data : &[u8]) -> bool {
        let signature = String::from_utf8(data[0..4].to_vec()).unwrap();
        return signature == "FILM";
    }

    /// Parses the passed slice of bytes, returning a `FILMHeader` object.
    ///
    /// If the supplied data doesn't appear to contain a FILM file, returns `Err`.
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

/// The FDSC chunk contains information about the streams inside the container;
/// it provides the information necessary to be able to decode the content.
pub struct FDSC {
    // Always 'FDSC'
    #[allow(dead_code)]
    signature: String,
    #[allow(dead_code)]
    length: u32,
    fourcc: String,
    /// The height of the video, in pixels.
    pub height: u32,
    /// The width of the video, in pixels.
    pub width: u32,
    /// The colour depth of the video's image. 24 is the only value that's been observed.
    pub bpp: u8,
    /// The number of channels of audio, typically 1 or 2.
    pub channels: u8,
    /// The bit depth of the audio stream; in practice this is always either 8 or 16.
    pub audio_resolution: u8,
    /// The type of compression used. 0, the default value, refers to uncompressed PCM, while 2 refers to CRI ADX.
    pub audio_compression: AudioCodec,
    /// The audio stream's sampling rate.
    pub audio_sampling_rate: u16,
}

impl FDSC {
    /// Parses the passed slice of bytes, returning an `FDSC` object.
    pub fn parse(data : &[u8]) -> FDSC {
        let signature_bytes = vec![
            data[0], data[1], data[2], data[3],
        ];
        let fourcc_bytes = vec![
            data[8], data[9], data[10], data[11],
        ];
        let audio_codec;
        match data[23] {
            0 => audio_codec = AudioCodec::PCM,
            2 => audio_codec = AudioCodec::ADX,
            _ => audio_codec = AudioCodec::Unknown,
        };

        return FDSC {
            signature: String::from_utf8(signature_bytes).unwrap(),
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]),
            fourcc: String::from_utf8(fourcc_bytes).unwrap(),
            height: uint32_from_bytes([data[12], data[13], data[14], data[15]]),
            width: uint32_from_bytes([data[16], data[17], data[18], data[19]]),
            bpp: data[20],
            channels: data[21],
            audio_resolution: data[22],
            audio_compression: audio_codec,
            audio_sampling_rate: uint16_from_bytes([data[24], data[25]]),
        };
    }

    /// Returns a string identifying the audio format. Valid return values are "pcm" and "adx".
    pub fn audio_codec(&self) -> &'static str {
        return self.audio_compression.name();
    }

    /// Parses the fourcc and returns a human-readable description of the video codec.
    pub fn human_readable_fourcc(&self) -> &'static str {
        if self.fourcc == "cvid" {
            return "Cinepak";
        } else {
            return "Raw video";
        }
    }
}

/// The STAB chunk contains the sample table.
/// This table contains a list of every sample in the file;
/// when parsing the file, you'll use the data in this struct
/// to determine how to actually read the audio and video streams.
pub struct STAB {
    // Always 'STAB'
    #[allow(dead_code)]
    signature: String,
    #[allow(dead_code)]
    length: u32,
    /// The number of "ticks per second" of video.
    /// This isn't precisely a framerate; instead, a given frame takes up
    /// a given number of ticks, and this ticks-per-second value is used
    /// to calculate a time interval between frames.
    /// See the [Multimedia Wiki documentation](https://wiki.multimedia.cx/index.php/Sega_FILM#FILM_Framerate_Calculation) for more information.
    pub framerate: u32,
    // Number of entries in the sample table
    #[allow(dead_code)]
    entries: u32,
    /// A vector containing Samples, which are references to the actual stream
    /// data in the FILM file.
    pub sample_table: Vec<Sample>,
}

impl STAB {
    /// Parses the passed slice of bytes, returning a `STAB` object.
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

/// Represents a single sample in the file.
/// Each sample within the FILM file is either audio or video; this data from the
/// sample table will tell you the location of the sample along with what kind of data it
/// contains and some basic metadata about it.
/// For the audio stream, this information and the information in the FDSC is enough to
/// parse the data after you've demuxed it.
/// For the video stream, you'll need some additional information from the Cinepak headers
/// which are contained in every video sample.
///
/// The offset in the sample data is relative to the end of the header;
/// you can use your FILMHeader's `length` to determine that offset.
/// For example, to extract this sample from the file's data:
///
/// ```
/// // assuming a FILMHeader named `header`, and the entire file's contents as `film_data`
/// let sample = header.stab.sample_table[0];
/// let absolute_sample_offset = header.length + sample.offset;
/// let sample_data = film_data[absolute_sample_offset..absolute_sample_offset + sample.length];
/// ```
pub struct Sample {
    /// Offset to the beginning of the sample's data.
    /// This is normally relative to the beginning of the sample data - that is,
    /// byte 0 is the first byte after the header ends.
    pub offset: usize,
    /// The length of this sample's data, in bytes.
    pub length: usize,
    info1: [u8; 4],
    #[allow(dead_code)]
    info2: [u8; 4],
}

impl Sample {
    /// Parses the passed slice of bytes, returning a `Sample` object.
    pub fn parse(data : &[u8]) -> Sample {
        return Sample {
            offset: uint32_from_bytes([data[0], data[1], data[2], data[3]]) as usize,
            length: uint32_from_bytes([data[4], data[5], data[6], data[7]]) as usize,
            info1: [data[8], data[9], data[10], data[11]],
            info2: [data[12], data[13], data[14], data[15]],
        }
    }

    /// Reads the metadata in this Sample to determine whether this sample contains audio.
    pub fn is_audio(&self) -> bool {
        return self.info1 == [255, 255, 255, 255];
    }

    /// Reads the metadata in this Sample to determine whether this sample contains video.
    pub fn is_video(&self) -> bool {
        return !self.is_audio();
    }

    /// Returns Some(true) if this frame is a keyframe. This is only relevant for video samples.
    /// Cinepak implements interframe compression; keyframes contain the entire frame,
    /// while subsequent non-key frames update only portions of the image.
    ///
    /// If this sample isn't video, returns None.
    pub fn is_keyframe(&self) -> Option<bool> {
        if !self.is_video() {
            return None;
        }

        let byte = uint32_from_bytes(self.info1);
        return Some((byte & (1 << 31)) == 0);
    }
}
