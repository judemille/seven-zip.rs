#![allow(clippy::module_name_repetitions)]

use std::io::{Read, Seek, SeekFrom};

use bitfield::{bitfield, Bit};
use byteorder::{ReadBytesExt, LE};

use crate::{
    error::ReadError,
    types::{read_7z_varint, Codec},
};

const END: u8 = 0x00;

pub struct MainHeader {
    additional_streams_info: Option<StreamsInfo>,
    main_streams_info: Option<StreamsInfo>,
    files: Option<FilesInfo>,
}

impl MainHeader {
    pub const IDENT: u8 = 0x01;
    pub const ARCHIVE_PROPERTIES: u8 = 0x02;
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        verify_ident(reader, Self::IDENT)?;
        let mut additional_streams_info: Option<StreamsInfo> = None;
        let mut main_streams_info: Option<StreamsInfo> = None;
        let mut files: Option<FilesInfo> = None;
        loop {
            let next_byte = reader.read_u8()?;
            match next_byte {
                StreamsInfo::ADDITIONAL_STREAMS => {
                    reader.seek(SeekFrom::Current(-1))?;
                    additional_streams_info = Some(StreamsInfo::try_read(
                        reader,
                        StreamsInfo::ADDITIONAL_STREAMS,
                    )?);
                }
                StreamsInfo::MAIN_STREAMS => {
                    reader.seek(SeekFrom::Current(-1))?;
                    main_streams_info =
                        Some(StreamsInfo::try_read(reader, StreamsInfo::MAIN_STREAMS)?);
                }
                FilesInfo::IDENT => {
                    reader.seek(SeekFrom::Current(-1))?;
                    files = Some(FilesInfo::try_read(reader)?);
                }
                Self::ARCHIVE_PROPERTIES => {
                    return Err(ReadError::UnsupportedFeature {
                        feat: "ArchiveProperties in Header".to_owned(),
                        reason: "I cannot find any actual information on what data this contains or how to implement it.".to_owned(),
                    })
                }
                END => break,
                _ => return Err(ReadError::Invalid7z),
            }
        }
        Ok(Self {
            additional_streams_info,
            main_streams_info,
            files,
        })
    }
}

pub struct StreamsInfo {
    pack: Option<PackInfo>,
    unpack: Option<UnpackInfo>,
    substream: Option<SubstreamInfo>,
}

impl StreamsInfo {
    pub const ADDITIONAL_STREAMS: u8 = 0x03;
    pub const MAIN_STREAMS: u8 = 0x04;
    pub fn try_read<R: Read + Seek>(reader: &mut R, ident: u8) -> Result<Self, ReadError> {
        verify_ident(reader, ident)?;
        let pack = PackInfo::try_read(reader).ok();
        let unpack = UnpackInfo::try_read(reader).ok();
        let substream = SubstreamInfo::try_read(reader).ok();
        verify_ident(reader, END)?;
        Ok(Self {
            pack,
            unpack,
            substream,
        })
    }
}

pub struct PackInfo {
    pack_position: u64,
    count_streams: u64,
    stream_sizes: Option<Vec<u64>>,
    stream_crcs: Option<Vec<Option<u32>>>,
}

impl PackInfo {
    pub const IDENT: u8 = 0x06;
    #[allow(clippy::cast_possible_truncation)]
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        verify_ident(reader, Self::IDENT)?;
        let pack_position = read_7z_varint(reader)?;
        let count_streams = read_7z_varint(reader)?;
        let tag = reader.read_u8()?;
        if count_streams == 0 {
            if tag == 0x00 {
                return Ok(Self {
                    pack_position,
                    count_streams,
                    stream_sizes: None,
                    stream_crcs: None,
                });
            } else if tag == 0x09 {
                if reader.read_u8()? != 0x00 {
                    return Err(ReadError::Invalid7z);
                }
                return Ok(Self {
                    pack_position,
                    count_streams,
                    stream_sizes: None,
                    stream_crcs: None,
                });
            }
            return Err(ReadError::Invalid7z);
        } else if tag != 0x09 {
            return Err(ReadError::Invalid7z);
        }
        let mut stream_sizes = Vec::<u64>::new();
        stream_sizes.reserve_exact(count_streams as usize);
        for _ in 0..count_streams {
            stream_sizes.push(read_7z_varint(reader)?);
        }
        verify_ident(reader, 0x0A)?; // CRC, Property ID
        let mut stream_crcs = Vec::<Option<u32>>::new();
        stream_crcs.reserve_exact(count_streams as usize);
        let defined = if reader.read_u8()? == 0 {
            let mut defined = Vec::new();
            defined.reserve_exact(count_streams as usize);
            let bytes = count_streams.div_ceil(8);
            let mut bytes = vec![0u8; bytes as usize].into_boxed_slice();
            reader.read_exact(&mut bytes)?;
            for bit in 0..count_streams {
                let byte = (bit / 8) as usize;
                let bit = (bit % 8) as usize;
                defined.push(bytes[byte].bit(bit));
            }
            defined
        } else {
            vec![true; count_streams as usize]
        };
        for is_defined in defined {
            if is_defined {
                let crc = reader.read_u32::<LE>()?;
                stream_crcs.push(Some(crc));
            } else {
                stream_crcs.push(None);
            }
        }
        verify_ident(reader, END)?;
        Ok(Self {
            pack_position,
            count_streams,
            stream_sizes: Some(stream_sizes),
            stream_crcs: Some(stream_crcs),
        })
    }
}

pub struct UnpackInfo {}

impl UnpackInfo {
    pub const IDENT: u8 = 0x07;
    pub const NUM_FOLDERS: u8 = 0x0B;
    #[allow(clippy::cast_possible_truncation)]
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        verify_ident(reader, Self::IDENT)?;
        verify_ident(reader, Self::NUM_FOLDERS)?;

        let num_folders = reader.read_u64::<LE>()?;
        let external: bool = reader.read_u8()? != 0;
        let mut folders: Vec<FolderInfo> = Vec::new();
        folders.reserve_exact(num_folders as usize);
        #[allow(clippy::redundant_else)]
        if external {
            return Err(ReadError::UnsupportedFeature {
                feat: "Non-inline folder information".to_owned(),
                reason: "This feature is more complex, and is being omitted in the initial implementation.".to_owned(),
            });
        } else {
            for _ in 0..num_folders {
                folders.push(FolderInfo::try_read(reader)?);
            }
        }

        verify_ident(reader, END)?;
        todo!()
    }
}

pub struct SubstreamInfo {}

impl SubstreamInfo {
    pub const IDENT: u8 = 0x08;
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        verify_ident(reader, Self::IDENT)?;
        todo!()
    }
}

pub struct FilesInfo {}

impl FilesInfo {
    pub const IDENT: u8 = 0x05;
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        verify_ident(reader, Self::IDENT)?;
        todo!()
    }
}

pub struct FolderInfo {
    coders: Vec<CoderInfo>,
    bind_pairs: Vec<(u64, u64)>,
    packed_stream_indices: Vec<u64>,
}

impl FolderInfo {
    #[allow(clippy::cast_possible_truncation)]
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        let num_coders = reader.read_u64::<LE>()?;
        let mut coders: Vec<CoderInfo> = Vec::new();
        coders.reserve_exact(num_coders as usize);
        let mut num_in_streams_total = 0;
        let mut num_out_streams_total = 0;
        for _ in 0..num_coders {
            let id_info = CoderIdInfo(reader.read_u8()?);
            let id_size: u8 = id_info.id_size();
            let codec_id = Codec::from(reader.read_uint::<LE>(id_size as usize)?);
            let num_in_streams = if id_info.complex() {
                reader.read_u64::<LE>()?
            } else {
                1
            };
            let num_out_streams = if id_info.complex() {
                reader.read_u64::<LE>()?
            } else {
                1
            };
            num_in_streams_total += num_in_streams;
            num_out_streams_total += num_out_streams;
            let mut properties: Vec<u8> = Vec::new();
            if id_info.exist_attrs() {
                let num_properties = reader.read_u64::<LE>()? as usize;
                properties.reserve_exact(num_properties);
                for _ in 0..num_properties {
                    properties.push(reader.read_u8()?);
                }
            }
            coders.push(CoderInfo {
                id_info,
                codec_id,
                num_in_streams,
                num_out_streams,
                properties,
            });
        }
        let num_bind_pairs = num_out_streams_total - 1;
        let mut bind_pairs: Vec<(u64, u64)> = Vec::new();
        bind_pairs.reserve_exact(num_bind_pairs as usize);
        for _ in 0..num_bind_pairs {
            bind_pairs.push((reader.read_u64::<LE>()?, reader.read_u64::<LE>()?));
        }
        let num_packed_streams = num_in_streams_total - num_bind_pairs;
        todo!()
    }
}

pub struct CoderInfo {
    id_info: CoderIdInfo,
    codec_id: Codec,
    num_in_streams: u64,
    num_out_streams: u64,
    properties: Vec<u8>,
}

bitfield! {
    pub struct CoderIdInfo(u8);
    impl Debug;
    u8;
    id_size, set_id_size: 3, 0;
    complex, set_complex: 4;
    exist_attrs, set_exist_attrs: 5;
    reserved, _: 6;
    /// This must be zero.
    more_alt, _: 7;
}

pub struct EncodedHeader {
    pack: Option<PackInfo>,
    unpack: Option<UnpackInfo>,
    substream: Option<SubstreamInfo>,
}

impl EncodedHeader {
    pub const IDENT: u8 = 0x17;
    pub fn try_read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReadError> {
        verify_ident(reader, Self::IDENT)?;
        let pack = PackInfo::try_read(reader).ok();
        let unpack = UnpackInfo::try_read(reader).ok();
        let substream = SubstreamInfo::try_read(reader).ok();
        Ok(Self {
            pack,
            unpack,
            substream,
        })
    }
}

#[inline]
fn verify_ident<R: Read + Seek>(reader: &mut R, ident: u8) -> Result<(), ReadError> {
    if reader.read_u8()? != ident {
        return Err(ReadError::Invalid7z);
    }
    Ok(())
}
