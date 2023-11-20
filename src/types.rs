use std::io::{Error as IoError, Read};

use byteorder::{ReadBytesExt, LE};
use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};

#[repr(u64)]
#[derive(Eq, PartialEq, FromPrimitive, IntoPrimitive, Debug)]
pub enum Codec {
    // Filters.
    Arm = 0x07,
    ArmT = 0x08,
    Arm64 = 0x0A,
    Bcj = 0x04,
    BcjArm = 0x0303_0501,
    BcjArmT = 0x0303_0701,
    BcjIA64 = 0x0303_0301,
    BcjPpc = 0x0303_0205,
    BcjSparc = 0x0303_0805,
    Delta = 0x03,
    IA64 = 0x06,
    PPC = 0x05,
    SPARC = 0x09,
    // Compression.
    Brotli = 0x04f7_1102,
    Bzip2 = 0x04_0202,
    Copy = 0x00,
    Deflate = 0x04_0108,
    Deflate64 = 0x04_0109,
    Lizard = 0x04f7_1106,
    LZ4 = 0x04f7_1104,
    LZMA = 0x03_0101,
    LZMA2 = 0x21,
    PPMd = 0x03_0401, // ?
    P7zBcj = 0x0303_0103,
    P7zBcj2 = 0x0303_011b,
    Zstd = 0x04f7_1101,
    // Cryptography.
    Aes128Ecb = 0x06f0_0100,
    Aes128Cbc = 0x06f0_0101,
    Aes128Cfb = 0x06f0_0102,
    Aes128Ofb = 0x06f0_0103,
    Aes128Ctr = 0x06f0_0104,
    Aes192Ecb = 0x06f0_0140,
    Aes192Cbc = 0x06f0_0141,
    Aes192Cfb = 0x06f0_0142,
    Aes192Ofb = 0x06f0_0143,
    Aes192Ctr = 0x06f0_0144,
    Aes256Ecb = 0x06f0_0180,
    Aes256Cbc = 0x06f0_0181,
    Aes256Cfb = 0x06f0_0182,
    Aes256Ofb = 0x06f0_0183,
    Aes256Ctr = 0x06f0_0184,
    ZipCrypto = 0x06f1_0101, // main zip crypt algo
    Rar29AES = 0x06f1_0303,  // AES-128 + modified SHA-1
    SevenZAES = 0x06f1_0701, // AES-256 + SHA-256
    #[num_enum(catch_all)]
    Unrecognized(u64) = 0x3F
}

pub(crate) fn read_7z_varint<R: Read>(reader: &mut R) -> Result<u64, IoError> {
    let first_byte = reader.read_u8()?;
    let extra_len: u8 = if first_byte & 0b1000_0000 == 0 {
        0
    } else if first_byte & 0b1100_0000 == 0b1000_0000 {
        1
    } else if first_byte & 0b1110_0000 == 0b1100_0000 {
        2
    } else if first_byte & 0b1111_0000 == 0b1110_0000 {
        3
    } else if first_byte & 0b1111_1000 == 0b1111_0000 {
        4
    } else if first_byte & 0b1111_1100 == 0b1111_1000 {
        5
    } else if first_byte & 0b1111_1110 == 0b1111_1100 {
        6
    } else if first_byte == 0b1111_1110 {
        7
    } else {
        8
    };
    if extra_len == 0 {
        return Ok(u64::from(first_byte));
    }
    if extra_len == 7 {
        return reader.read_uint::<LE>(7usize);
    }
    if extra_len == 8 {
        return reader.read_u64::<LE>();
    }
    let extra = reader.read_uint::<LE>(extra_len as usize)?;
    let mask = match extra_len {
        1 => 0b0111_1111,
        2 => 0b0011_1111,
        3 => 0b0001_1111,
        4 => 0b0000_1111,
        5 => 0b0000_0111,
        6 => 0b0000_0011,
        _ => unreachable!(),
    };
    let first_byte = u64::from(first_byte & mask);
    let value = (first_byte << (8 * extra_len)) + extra;
    Ok(value)
}
