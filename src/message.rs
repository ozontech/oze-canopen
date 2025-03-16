use crate::proto::CobId;
use socketcan::{CanDataFrame, EmbeddedFrame};
use tokio::time::Instant;

/// Represents a node identifier in the CAN network.
pub type NodeId = u8;

/// Enum for different string formats for RX messages.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RxMessageToStringFormat {
    /// Binary format.
    Binary,
    /// Hexadecimal format.
    Hex,
    /// ASCII format.
    Ascii,
    /// UTF-8 format.
    Utf8,
}

/// Struct representing a received message.
///
/// # Examples
///
/// ```
/// use oze_canopen::{RxMessage, CanDataFrame, RxMessageToStringFormat};
///
/// let frame = CanDataFrame::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
/// if let Some(rx_msg) = RxMessage::from(frame) {
///     println!("{}", rx_msg.data_to_string(RxMessageToStringFormat::Hex)); // Output: "DE AD BE EF"
///     println!("{}", rx_msg.cob_id_to_string()); // Output: "123"
/// }
/// ```
#[derive(Clone, Debug, Copy)]
pub struct RxMessage {
    /// Timestamp when the message was received.
    pub timestamp: Instant,
    /// COB ID of the message.
    pub cob_id: CobId,
    /// Data payload of the message.
    pub data: [u8; 8],
    /// Data length code (number of bytes in the data payload).
    pub dlc: usize,
}

/// Converts a byte to a lossy ASCII string representation.
///
/// # Examples
///
/// ```
/// use oze_canopen::u8_to_ascii_lossy;
///
/// let ascii_str = u8_to_ascii_lossy(65); // 'A'
/// let non_ascii_str = u8_to_ascii_lossy(200); // '.'
/// println!("{}", ascii_str); // Output: "A"
/// println!("{}", non_ascii_str); // Output: "."
/// ```
pub(crate) fn u8_to_ascii_lossy(byte: u8) -> String {
    if byte.is_ascii() && !byte.is_ascii_control() {
        byte.escape_ascii().to_string()
    } else {
        ".".to_owned()
    }
}

impl RxMessage {
    /// Creates an `RxMessage` from a `CanDataFrame`.
    ///
    /// # Arguments
    ///
    /// * `frame` - The CAN data frame.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `RxMessage` if the frame ID is standard, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::{RxMessage, CanDataFrame};
    ///
    /// let frame = CanDataFrame::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
    /// if let Some(rx_msg) = RxMessage::from(frame) {
    ///     println!("{:?}", rx_msg);
    /// }
    /// ```
    pub fn from(frame: CanDataFrame) -> Option<Self> {
        let socketcan::Id::Standard(id) = frame.id() else {
            eprintln!("EEE");
            return None;
        };

        let mut data = [0u8; 8];
        let frame_data = frame.data();
        let len = frame_data.len().min(8);
        data[..len].copy_from_slice(&frame_data[..len]);
        Some(RxMessage {
            timestamp: Instant::now(),
            cob_id: id.as_raw(),
            data,
            dlc: frame.dlc(),
        })
    }

    /// Converts the data payload of the message to a string based on the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format in which to convert the data payload.
    ///
    /// # Returns
    ///
    /// A `String` representation of the data payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::{RxMessage, CanDataFrame, RxMessageToStringFormat};
    ///
    /// let frame = CanDataFrame::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
    /// if let Some(rx_msg) = RxMessage::from(frame) {
    ///     let hex_str = rx_msg.data_to_string(RxMessageToStringFormat::Hex);
    ///     println!("{}", hex_str); // Output: "DE AD BE EF"
    /// }
    /// ```
    pub fn data_to_string(&self, format: RxMessageToStringFormat) -> String {
        if format == RxMessageToStringFormat::Utf8 {
            return String::from_utf8_lossy(&self.data).to_string();
        }

        let mut out = String::with_capacity(self.data.len() * 9);
        for (i, d) in self.data.iter().enumerate() {
            if i >= self.dlc {
                break;
            }

            if i == 0 {
                if format == RxMessageToStringFormat::Binary {
                    out.insert_str(0, &format!("{:08b}", d));
                } else if format == RxMessageToStringFormat::Ascii {
                    out.insert_str(0, &u8_to_ascii_lossy(*d));
                } else {
                    out.insert_str(0, &format!("{:02X}", d));
                }
            } else if format == RxMessageToStringFormat::Binary {
                out.insert_str((i - 1) * 9 + 8, &format!(" {:08b}", d));
            } else if format == RxMessageToStringFormat::Ascii {
                out.insert_str(i, &u8_to_ascii_lossy(*d));
            } else {
                out.insert_str((i - 1) * 3 + 2, &format!(" {:02X}", d));
            }
        }
        out
    }

    /// Converts the COB ID of the message to a string.
    ///
    /// # Returns
    ///
    /// A `String` representation of the COB ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::{RxMessage, CanDataFrame};
    ///
    /// let frame = CanDataFrame::new(0x123, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
    /// if let Some(rx_msg) = RxMessage::from(frame) {
    ///     let cob_id_str = rx_msg.cob_id_to_string();
    ///     println!("{}", cob_id_str); // Output: "123"
    /// }
    /// ```
    pub fn cob_id_to_string(&self) -> String {
        format!("{:03X}", self.cob_id)
    }
}
