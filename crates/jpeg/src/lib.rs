use serde::Deserialize;
use serde::Serialize;

/// Function `identify` check whether the chunk of data looks like JPEG structure.
/// If so, then return recognized image dimension (in pixels).
pub fn identify(bytes: &[u8]) -> Option<ImageSize> {
    let mut bytes = bytes;

    let mut offset = 0;
    while !bytes.is_empty() {
        let chunk = Chunk::from_bytes(bytes)?;

        if offset == 0 && !matches!(chunk.typ, ChunkType::StartOfImage) {
            // The StartOfImage chunk has to be the very first one
            return None;
        }

        const CHUNK_HEADER_SIZE: usize = 2;
        const CHUNK_SIZE_LEN: usize = 2;

        // 1 byte  - precision
        // 2 bytes - height
        // 2 bytes - width
        // 1 byte  - components
        const MIN_BYTES: usize = 6;

        if matches!(
            chunk.typ,
            ChunkType::Baseline | ChunkType::Progressive | ChunkType::ExtendedSequential
        ) {
            let (_, bytes) = bytes.split_at_checked(CHUNK_HEADER_SIZE + CHUNK_SIZE_LEN)?;
            if bytes.len() < MIN_BYTES {
                return None;
            }

            let img_size = ImageSize {
                width: mk_word(bytes[3], bytes[4]),
                height: mk_word(bytes[1], bytes[2]),
            };

            if img_size.width > 0 && img_size.height > 0 {
                return Some(img_size);
            } else {
                return None;
            }
        }

        let skip = CHUNK_HEADER_SIZE + chunk.size as usize;
        let Some((_, tail)) = bytes.split_at_checked(skip) else {
            // encountered some garbage
            break;
        };

        bytes = tail;
        offset += skip;
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
struct Chunk {
    pub typ: ChunkType,
    pub size: u16,
}

impl Chunk {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 2 {
            return None;
        }

        if bytes[0] != 0xff {
            return None;
        }

        let typ = ChunkType::from_byte(bytes[1])?;
        let size = if typ.is_dataless() {
            0
        } else {
            if bytes.len() < 4 {
                return None;
            }

            mk_word(bytes[2], bytes[3])
        };

        Some(Chunk { typ, size })
    }
}

#[inline]
fn mk_word(b0: u8, b1: u8) -> u16 {
    let b0 = b0 as u16;
    let b1 = b1 as u16;

    (b0 << 8) | b1
}

#[derive(Debug)]
enum ChunkType {
    Comment,
    Baseline,
    ExtendedSequential,
    Progressive,
    HuffmanTable,
    Exif,
    IccProfile,
    QuantizationTable,
    RestartInterval,
    StartOfImage,
    EndOfImage,
    Restart,
    Jfif,
}

impl ChunkType {
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            0xfe => Some(Self::Comment),
            0xc0 => Some(Self::Baseline),
            0xc1 => Some(Self::ExtendedSequential),
            0xc2 => Some(Self::Progressive),
            0xc4 => Some(Self::HuffmanTable),
            0xdb => Some(Self::QuantizationTable),
            0xd8 => Some(Self::StartOfImage),
            0xd9 => Some(Self::EndOfImage),
            0xdd => Some(Self::RestartInterval),
            0xe0 => Some(Self::Jfif),
            0xe1 => Some(Self::Exif),
            0xe2 => Some(Self::IccProfile),
            0xd0..=0xd7 => Some(Self::Restart),
            _ => None,
        }
    }

    #[inline]
    const fn is_dataless(&self) -> bool {
        matches!(
            self,
            ChunkType::StartOfImage | ChunkType::EndOfImage | ChunkType::Restart
        )
    }
}
