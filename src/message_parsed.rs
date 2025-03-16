use crate::{
    canopen::{NodeId, RxMessage},
    proto::CobId,
};

/// Enumeration representing the type of a received CANopen message.
///
/// This enum is used to classify the different types of CANopen messages that can be received.
/// Each variant corresponds to a specific type of CANopen message, as indicated by the COB-ID
/// (Communication Object Identifier).
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RxMessageType {
    /// Service Data Object Transmit (SDO Tx)
    ///
    /// Represents an SDO message sent from the server (node) to the client.
    SdoTx,

    /// Service Data Object Receive (SDO Rx)
    ///
    /// Represents an SDO message sent from the client to the server (node).
    SdoRx,

    /// Process Data Object (PDO)
    ///
    /// Represents any PDO message or Timestamp with COB-ID in the range 0x100 to 0x57F.
    Pdo,

    /// Synchronization (SYNC)
    ///
    /// Represents a SYNC message with COB-ID 0x80.
    Sync,

    /// Network Management (NMT)
    ///
    /// Represents an NMT message.
    Nmt,

    /// Layer Setting Services (LSS)
    ///
    /// Represents an LSS message.
    Lss,

    /// Node Guarding
    ///
    /// Represents a Node Guarding message.
    Guarding,

    /// Emergency (EMCY)
    ///
    /// Represents an EMCY message.
    Emcy,

    /// Unknown Message Type
    ///
    /// Represents a message type that could not be determined.
    Unknown,
}

/// Struct representing a parsed received CANopen message, including its type and optional node ID.
#[derive(Clone, Debug, Copy)]
pub struct RxMessageParsed {
    /// The parsed type of the received message.
    pub parsed_type: RxMessageType,
    /// The parsed node ID from the message, if applicable.
    pub parsed_node_id: Option<NodeId>,
    /// The original received message.
    pub msg: RxMessage,
}

impl RxMessageParsed {
    /// Creates a new `RxMessageParsed` instance by parsing the given received message.
    ///
    /// # Arguments
    ///
    /// * `msg` - The received CANopen message to be parsed.
    ///
    /// # Returns
    ///
    /// A new `RxMessageParsed` instance with the parsed type and node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::RxMessageParsed;
    ///
    /// let rx_message = RxMessage { /* fields */ };
    /// let parsed_message = RxMessageParsed::new(rx_message);
    /// ```
    pub fn new(msg: RxMessage) -> Self {
        Self {
            parsed_type: Self::parse_type(msg.cob_id),
            parsed_node_id: Self::parse_node_id(msg.cob_id),
            msg,
        }
    }

    /// Parses the type of a received message based on its COB ID.
    ///
    /// # Arguments
    ///
    /// * `cob_id` - The COB ID of the received message.
    ///
    /// # Returns
    ///
    /// The parsed type of the message as an `RxMessageType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::RxMessageParsed;
    ///
    /// let message_type = RxMessageParsed::parse_type(0x580);
    /// assert_eq!(message_type, RxMessageType::SdoTx);
    /// ```
    pub fn parse_type(cob_id: CobId) -> RxMessageType {
        match cob_id {
            0x0..=0x7F => RxMessageType::Nmt,
            0x80 => RxMessageType::Sync,
            0x100..=0x57F => RxMessageType::Pdo,
            0x580..=0x5FF => RxMessageType::SdoTx,
            0x600..=0x67F => RxMessageType::SdoRx,
            0x81..=0xFF => RxMessageType::Emcy,
            0x680..=0x6FF => RxMessageType::Unknown,
            0x700..=0x77F => RxMessageType::Guarding,
            0x780..=0x7FF => RxMessageType::Lss,
            0x800.. => RxMessageType::Unknown,
        }
    }

    /// Parses the node ID from a received message based on its COB ID.
    ///
    /// # Arguments
    ///
    /// * `cob_id` - The COB ID of the received message.
    ///
    /// # Returns
    ///
    /// The parsed node ID as an `Option<NodeId>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::RxMessageParsed;
    ///
    /// let node_id = RxMessageParsed::parse_node_id(0x580);
    /// assert_eq!(node_id, Some(0x0));
    /// ```
    pub fn parse_node_id(cob_id: CobId) -> Option<NodeId> {
        match cob_id {
            0x00..=0x80 => None,                                 // NMT, SYNC
            0x81..=0xFF => NodeId::try_from(cob_id - 0x80).ok(), // EMCY
            0x100..=0x57F => {
                let mut node_id = cob_id;
                while node_id >= 0x80 {
                    node_id -= 0x80;
                }
                NodeId::try_from(node_id).ok()
            } // PDO
            0x580..=0x5FF => NodeId::try_from(cob_id - 0x580).ok(), // SDO Transmit
            0x600..=0x67F => NodeId::try_from(cob_id - 0x600).ok(), // SDO Receive
            0x680..=0x6FF => None,                               // ???
            0x700..=0x77F => NodeId::try_from(cob_id - 0x700).ok(), // Node Guarding
            0x780..=0x7FF => None,                               // Lss
            0x800.. => None,                                     // not possible ???
        }
    }
}

impl RxMessageType {
    pub fn to_string(&self) -> &str {
        match &self {
            RxMessageType::SdoTx => "↓SDO S->C",
            RxMessageType::SdoRx => "↑SDO C->S",
            RxMessageType::Pdo => "PDO",
            RxMessageType::Sync => "SYNC",
            RxMessageType::Nmt => "NMT",
            RxMessageType::Unknown => "?",
            RxMessageType::Emcy => "EMCY",
            RxMessageType::Guarding => "Guard",
            RxMessageType::Lss => "LSS",
        }
    }
}
