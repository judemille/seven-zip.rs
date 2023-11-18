use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};
use crc::Crc;

use crate::error::ReadError;

pub struct Reader<R: Read + Seek> {
    stream: R,
    crc: Crc<u32>,
    size: usize,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(mut stream: R) -> Result<Self, ReadError> {
        stream.seek(SeekFrom::Start(0))?;

        {
            let mut sig = [0u8; 8];
            stream.read_exact(&mut sig)?;
            if sig[0..6] != [b'7', b'z', 0xbc, 0xaf, 0x27, 0x1c] || sig[6] != 0x00 || sig[7] != 0x04
            {
                return Err(ReadError::Invalid7z);
            }
        }

        let expected_crc = stream.read_u32::<LE>()?;

        let mut final_sig_head = [0u8; 20];
        stream.read_exact(&mut final_sig_head)?;

        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        if crc.checksum(&final_sig_head) != expected_crc {
            return Err(ReadError::CrcInvalid)
        }

        let mut final_sig_head_: &[u8] = &final_sig_head; // Allow use of byteorder::ReadBytesExt and consuming bytes.
        let next_head_offset = final_sig_head_.read_u64::<LE>()? + 32; // File gives offset relative to here. Add 32 to make an absolute offset.
        let next_head_size = final_sig_head_.read_u64::<LE>()?;
        let next_head_crc = final_sig_head_.read_u32::<LE>()?;

        stream.seek(SeekFrom::Start(next_head_offset))?;
        let header_type = stream.read_u8()?;
        todo!()
    }
}
