use binrw::{binrw, BinRead, BinWrite};

/// Represents the NMT command specifiers.
#[derive(Debug, BinWrite, BinRead, Clone, Copy, PartialEq, Eq)]
#[brw(repr=u8)]
pub enum NmtCommandSpecifier {
    /// Start the remote node.
    StartRemoteNode = 0x01,

    /// Stop the remote node.
    StopRemoteNode = 0x02,

    /// Enter pre-operational state.
    EnterPreOperational = 0x80,

    /// Reset the node.
    ResetNode = 0x81,

    /// Reset the communication.
    ResetCommunication = 0x82,
}

/// Represents an NMT command.
#[binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[brw(little)]
pub struct NmtCommand {
    /// The command specifier indicating the action to be taken.
    pub command_specifier: NmtCommandSpecifier,

    /// The node ID to which the command is directed.
    pub node_id: u8,
}

impl NmtCommand {
    /// Creates a new NMT command.
    ///
    /// # Arguments
    ///
    /// * `command_specifier` - The command specifier indicating the action to be taken.
    /// * `node_id` - The node ID to which the command is directed.
    ///
    /// # Examples
    ///
    /// ```
    /// let command = NmtCommand::new(NmtCommandSpecifier::StartRemoteNode, 1);
    /// ```
    pub fn new(command_specifier: NmtCommandSpecifier, node_id: u8) -> Self {
        Self {
            command_specifier,
            node_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinReaderExt;
    use std::io::Cursor;

    #[test]
    fn test_nmt_command_specifier_serialization() {
        let specifier = NmtCommandSpecifier::StartRemoteNode;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        specifier.write_le(&mut cursor).unwrap();
        assert_eq!(buffer, vec![0x01]);
    }

    #[test]
    fn test_nmt_command_specifier_deserialization() {
        let data = [0x80u8];
        let mut cursor = Cursor::new(&data);
        let specifier: NmtCommandSpecifier = cursor.read_le().unwrap();
        assert_eq!(specifier, NmtCommandSpecifier::EnterPreOperational);
    }

    #[test]
    fn test_nmt_command_serialization() {
        let command = NmtCommand::new(NmtCommandSpecifier::ResetNode, 5);
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        command.write(&mut cursor).unwrap();
        assert_eq!(buffer, vec![0x81, 0x05]);
    }

    #[test]
    fn test_nmt_command_deserialization() {
        let data = [0x82u8, 0x01];
        let mut cursor = Cursor::new(&data);
        let command: NmtCommand = cursor.read_le().unwrap();
        assert_eq!(
            command.command_specifier,
            NmtCommandSpecifier::ResetCommunication
        );
        assert_eq!(command.node_id, 1);
    }

    #[test]
    fn test_nmt_command_new() {
        let command = NmtCommand::new(NmtCommandSpecifier::StopRemoteNode, 10);
        assert_eq!(
            command.command_specifier,
            NmtCommandSpecifier::StopRemoteNode
        );
        assert_eq!(command.node_id, 10);
    }
}
