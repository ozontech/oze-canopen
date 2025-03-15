use super::sdo_abort::AbortCode;
use binrw::{binrw, BinRead, BinResult, BinWrite};
use bitflags::bitflags;

bitflags! {
    /// Command specifiers and flags for SDO requests.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        /// toggle bit. This bit must alternate for each subsequent segment that is uploaded. The first segment will have the toggle-bit set to 0. The toggle bit will be equal for the request and the response message.
        const TOGGLE_BIT = 0b0001_0000;

        /// Client command specifier or server command specifier mask.
        const S_3 = 0b0110_0000;
        const S_5 = 0b1010_0000;
        const S_6 = 0b1100_0000;
        const S_0 = 0b0000_0000;
        const S_1 = 0b0010_0000;
        const S_2 = 0b0100_0000;
        const S_4 = 0b1000_0000;
        const S_MASK = 0b1110_0000;


        /// Transfer type flag.
        /// 0: normal transfer
        /// 1: expedited transfer
        const EXPEDITED = 0b0000_0010;

        /// Size indicator flag.
        /// 0: data set size is not indicated
        /// 1: data set size is indicated
        const SIZE_SPECIFIED = 0b0000_0001;

        /// Only valid for SDO segmented download
        /// 0: no more segments left
        /// 1: next segments will follow
        const CONTINUE = 0b0000_0001;

        /// Only valid if e = 1 and s = 1, otherwise 0. If valid it indicates the number of bytes in d that do not contain data. Bytes [8-N, 7] do not contain segment data.
        const N_3 = 0b0000_1100;
        const N_0 = 0b0000_0000;
        const N_1 = 0b0000_0100;
        const N_2 = 0b0000_1000;
        const N_MASK = 0b0000_1100;

        /// Only valid for SDO segment download. Bytes [8-N, 7] do not contain segment data.
        const N8_3 = 0b0000_0110;
        const N8_6 = 0b0000_1100;
        const N8_0 = 0b0000_0000;
        const N8_1 = 0b0000_0010;
        const N8_2 = 0b0000_0100;
        const N8_4 = 0b0000_1000;
        const N8_5 = 0b0000_1010;
        const N8_MASK = 0b0000_1110;

    }
}

impl Flags {
    #[binrw::parser(reader)]
    fn parse() -> BinResult<Flags> {
        let byte = u8::read(reader)?;
        Ok(Flags::from_bits_truncate(byte))
    }

    #[binrw::writer(writer)]
    fn write(cmd: &Flags) -> BinResult<()> {
        cmd.bits().write(writer)?;
        Ok(())
    }
}

/// Represents the response to an Upload Segment SDO command.
#[binrw]
#[derive(Debug, Clone)]
#[br(import_raw(cmd:Flags))]
#[brw(assert(data.len() <= 7))]
pub struct SegmentData {
    #[br(count(7 - ((cmd & Flags::N8_MASK).bits() >> 1)))]
    pub data: Vec<u8>,
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import_raw(cmd:Flags))]
pub enum UploadResponseData {
    // e = 0, s = 1: d contains specified number of bytes to be uploaded
    #[br(pre_assert(!cmd.contains(Flags::EXPEDITED) && cmd.contains(Flags::SIZE_SPECIFIED)))]
    Size(u32),
    // e = 1, s = 0: d contains unspecified number of bytes. Not supported
    #[br(pre_assert(cmd.contains(Flags::EXPEDITED) && !cmd.contains(Flags::SIZE_SPECIFIED)))]
    ExpeditedWithoutSize,
    // e = 1, s = 1: d contains the data of length 4-n to be uploaded
    #[br(pre_assert(cmd.contains(Flags::EXPEDITED) && cmd.contains(Flags::SIZE_SPECIFIED)))]
    DataExpedited(
        #[br(count(4 - ((cmd & Flags::N_MASK).bits() >> 2)))]
        #[brw(assert(self_0.len() <= 4))]
        Vec<u8>,
    ),
    // e = 0, s = 0: d is reserved for further use. Not supported
    #[br(pre_assert(!cmd.contains(Flags::SIZE_SPECIFIED) && !cmd.contains(Flags::EXPEDITED)))]
    E0S0NotSupported,
}

/// Represents the response to an Initiate Upload SDO command.
#[binrw]
#[derive(Debug, Clone)]
#[br(import_raw(cmd:Flags))]
pub struct UploadResponse {
    pub index: u16,
    pub subindex: u8,
    #[br(args_raw=cmd)]
    pub data: UploadResponseData,
}

/// Represents the response to an Abort SDO command.
#[binrw]
#[derive(Debug, Clone)]
pub struct AbortResponse {
    pub index: u16,
    pub subindex: u8,
    pub reason: AbortCode,
}

/// Represents the response to an Initiate Download SDO command.
#[binrw]
#[derive(Debug, Clone)]
pub struct DownloadResponse {
    pub index: u16,
    pub subindex: u8,
    pub size: u32,
}

/// Represents different types of SDO responses.
#[derive(Debug, BinRead, Clone)]
#[br(import_raw(cmd:Flags))]
pub enum ResponseData {
    /// Response for the Initiate Download command.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_3))]
    Download(DownloadResponse),

    /// Response for the Download Segment command.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_1))]
    DownloadSegment(#[br(args_raw=cmd)] SegmentData),

    /// Response for the Initiate Upload command.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_2))]
    Upload(#[br(args_raw=cmd)] UploadResponse),

    /// Response for the Upload Segment command.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_0))]
    UploadSegment(#[br(args_raw=cmd)] SegmentData),

    /// Response for the Abort command.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_4))]
    Abort(AbortResponse),
}

/// Represents a server response to an SDO request.
#[derive(Debug, BinRead, Clone)]
#[br(little)]
pub struct SdoResponse {
    /// Command flags.
    #[br(parse_with = Flags::parse)]
    pub cmd: Flags,

    /// Specific server response data.
    #[br(args_raw = cmd)]
    pub resp: ResponseData,
}

/// Represents the initial SDO request with size information.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct InitiateRequest {
    pub index: u16,
    pub subindex: u8,
    pub size: u32,
}

/// Represents the initial SDO request with data.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
#[br(import_raw(cmd:Flags))]
pub struct DownloadExpeditedRequest {
    pub index: u16,
    pub subindex: u8,
    #[br(count(4 - ((cmd & Flags::N_MASK).bits() >> 2)))]
    #[brw(assert(data.len() <= 4))]
    pub data: Vec<u8>,
}

/// Represents the different types of SDO requests.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
#[br(import_raw(cmd:Flags))]
pub enum SdoRequestData {
    /// Initial request with size information.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_2))]
    InitiateUpload(InitiateRequest),

    /// Initial request without data, with size.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_1 && !cmd.contains(Flags::EXPEDITED)))]
    InitiateDownload(InitiateRequest),

    /// Initial request with data.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_1 && cmd.contains(Flags::EXPEDITED)))]
    InitiateDownloadExpedited(#[br(args_raw=cmd)] DownloadExpeditedRequest),

    /// Segmented requests.
    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_3))]
    UploadSegment([u8; 7]),

    #[br(pre_assert(cmd & Flags::S_MASK == Flags::S_0))]
    DownloadSegment(#[br(args_raw=cmd)] SegmentData),
}

/// Represents a server request to an SDO server.
#[binrw]
#[derive(Debug, Clone)]
#[brw(little)]
pub struct SdoRequest {
    /// Command flags.
    #[br(parse_with = Flags::parse)]
    #[bw(write_with = Flags::write)]
    pub cmd: Flags,

    /// Specific server response data.
    #[br(args_raw = cmd)]
    pub req: SdoRequestData,
}

impl SdoRequest {
    /// Creates a new SDO request to initiate an upload.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the object dictionary entry.
    /// * `subindex` - The subindex of the object dictionary entry.
    ///
    /// # Examples
    ///
    /// ```
    /// let request = SdoRequest::initiate_upload(0x2000, 0x01);
    /// ```
    pub fn initiate_upload(index: u16, subindex: u8) -> Self {
        Self {
            cmd: Flags::S_2,
            req: SdoRequestData::InitiateUpload(InitiateRequest {
                index,
                subindex,
                size: 0,
            }),
        }
    }

    /// Creates a new SDO request to upload a segment.
    ///
    /// # Arguments
    ///
    /// * `toggle` - The toggle bit value.
    ///
    /// # Examples
    ///
    /// ```
    /// let request = SdoRequest::upload_segment(true);
    /// ```
    pub fn upload_segment(toggle: bool) -> Self {
        Self {
            cmd: if toggle {
                Flags::S_3 | Flags::TOGGLE_BIT
            } else {
                Flags::S_3
            },
            req: SdoRequestData::UploadSegment([0; 7]),
        }
    }

    /// Creates a new SDO request to initiate a download with size information.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the object dictionary entry.
    /// * `subindex` - The subindex of the object dictionary entry.
    /// * `size` - The size of the data to be downloaded.
    ///
    /// # Examples
    ///
    /// ```
    /// let request = SdoRequest::initiate_download(0x2000, 0x01, 128);
    /// ```
    pub fn initiate_download(index: u16, subindex: u8, size: usize) -> Self {
        Self {
            cmd: Flags::SIZE_SPECIFIED | Flags::S_1,
            req: SdoRequestData::InitiateDownload(InitiateRequest {
                index,
                subindex,
                size: u32::try_from(size).unwrap(),
            }),
        }
    }

    /// Creates a new SDO request to initiate an expedited download.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the object dictionary entry.
    /// * `subindex` - The subindex of the object dictionary entry.
    /// * `src` - The data to be downloaded.
    ///
    /// # Examples
    ///
    /// ```
    /// let data = [0x12, 0x34, 0x56, 0x78];
    /// let request = SdoRequest::initiate_download_expedited(0x2000, 0x01, &data);
    /// ```
    pub fn initiate_download_expedited(index: u16, subindex: u8, src: &[u8]) -> Self {
        let mut data: [u8; 4] = [0; 4];
        let len = src.len().min(4);
        data[..len].copy_from_slice(&src[..len]);

        Self {
            cmd: Flags::S_1
                | Flags::EXPEDITED
                | Flags::SIZE_SPECIFIED
                | match src.len() {
                    1 => Flags::N_3,
                    2 => Flags::N_2,
                    3 => Flags::N_1,
                    4 => Flags::N_0,
                    _ => panic!("wrong size"),
                },
            req: SdoRequestData::InitiateDownloadExpedited(DownloadExpeditedRequest {
                data: data.to_vec(),
                index,
                subindex,
            }),
        }
    }

    /// Creates a new SDO request to download a segment.
    ///
    /// # Arguments
    ///
    /// * `toggle` - The toggle bit value.
    /// * `src` - The data to be downloaded.
    ///
    /// # Examples
    ///
    /// ```
    /// let data = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD];
    /// let request = SdoRequest::download_segment(true, &data);
    /// ```
    pub fn download_segment(toggle: bool, src: &[u8]) -> Self {
        let mut data: [u8; 7] = [0; 7];
        let len = src.len().min(7);
        data[..len].copy_from_slice(&src[..len]);

        let mut cmd = Flags::S_0;

        if toggle {
            cmd |= Flags::TOGGLE_BIT;
        }

        cmd |= match len {
            1 => Flags::N8_6,
            2 => Flags::N8_5,
            3 => Flags::N8_4,
            4 => Flags::N8_3,
            5 => Flags::N8_2,
            6 => Flags::N8_1,
            7 => Flags::N8_0,
            _ => Flags::N8_0,
        };

        Self {
            cmd,
            req: SdoRequestData::DownloadSegment(SegmentData {
                data: data.to_vec(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinReaderExt;
    use std::io::Cursor;

    #[test]
    fn test_flags_parse() {
        let data = 0b1010_0000u8;
        let parsed_flags: Flags = Flags::from_bits_truncate(data);
        assert_eq!(parsed_flags, Flags::S_5);
        assert_ne!(parsed_flags, Flags::S_3);
        assert_ne!(parsed_flags, Flags::S_1);
        assert_ne!(parsed_flags, Flags::S_2);

        let data = 0b1010_0001u8;
        let parsed_flags: Flags = Flags::from_bits_truncate(data);
        assert_ne!(parsed_flags, Flags::S_5);
        assert_ne!(parsed_flags, Flags::S_3);
        assert_ne!(parsed_flags, Flags::S_1);
        assert_ne!(parsed_flags, Flags::S_2);

        let parsed_flags = parsed_flags & Flags::S_MASK;
        assert_eq!(parsed_flags, Flags::S_5);
        assert_ne!(parsed_flags, Flags::S_1);

        assert_eq!(Flags::N8_1, Flags::EXPEDITED);
    }

    #[test]
    fn test_flags_write() {
        let flags = Flags::S_3 | Flags::EXPEDITED;
        let val = flags.bits().to_le();
        assert_eq!(val, 0b0110_0010u8);
    }

    #[test]
    fn test_server_response_initiate_download() {
        let data = [
            0b0110_0000, // Flags::S_3
            0x34,        // index
            0x12,
            0x01, // subindex
            0x78,
            0x56,
            0x34,
            0x12, // size
        ];
        let mut cursor = Cursor::new(&data);
        let response: SdoResponse = cursor.read_le().unwrap();
        if let ResponseData::Download(resp) = response.resp {
            assert_eq!(resp.index, 0x1234);
            assert_eq!(resp.subindex, 0x01);
            assert_eq!(resp.size, 0x12345678);
        } else {
            panic!("Expected InitiateDownload response");
        }
    }

    #[test]
    fn test_initiate_upload_request() {
        let request = SdoRequest::initiate_upload(0x2000, 0x01);
        if let SdoRequestData::InitiateUpload(req) = request.req {
            assert_eq!(req.index, 0x2000);
            assert_eq!(req.subindex, 0x01);
            assert!(request.cmd.contains(Flags::S_2));
        } else {
            panic!("Expected Initial request");
        }
    }

    #[test]
    fn test_initiate_download_request() {
        let request = SdoRequest::initiate_download(0x2000, 0x01, 0x4);
        if let SdoRequestData::InitiateDownload(req) = request.req {
            assert_eq!(req.index, 0x2000);
            assert_eq!(req.subindex, 0x01);
            assert_eq!(req.size, 0x04);
            assert!(request.cmd.contains(Flags::S_1));
        } else {
            panic!("Expected Initial request");
        }
    }
}
