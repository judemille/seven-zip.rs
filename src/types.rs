pub enum Codec {
    Bcj = 0x04,
    BcjArm = 0x0303_0501,
    BcjArmT = 0x0303_0701,
    BcjIA64 = 0x0303_0301,
    BcjPpc = 0x0303_0205,
    BcjSparc = 0x0303_0805,
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
    Zstd = 0x04f7_1101
}

pub enum CryptCodec {
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
    Rar29AES = 0x06f1_0303, // AES-128 + modified SHA-1
    SevenZAES = 0x06f1_0701 // AES-256 + SHA-256
}