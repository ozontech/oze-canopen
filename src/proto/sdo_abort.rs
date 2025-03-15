use std::fmt;

use binrw::binrw;

/// Represents various abort codes and their descriptions.
#[binrw]
#[derive(Debug, Clone)]
#[brw(repr = u32)]
pub enum AbortCode {
    ToggleBitNotAlternated = 0x05030000,
    SdoProtocolTimedOut = 0x05040000,
    ClientServerCommandSpecifierInvalid = 0x05040001,
    InvalidBlockSize = 0x05040002,
    InvalidSequenceNumber = 0x05040003,
    CrcError = 0x05040004,
    OutOfMemory = 0x05040005,
    UnsupportedAccessToObject = 0x06010000,
    AttemptToReadWriteOnlyObject = 0x06010001,
    AttemptToWriteReadOnlyObject = 0x06010002,
    ObjectDoesNotExistInObjectDictionary = 0x06020000,
    ObjectCannotBeMappedToPdo = 0x06040041,
    NumberAndLengthOfObjectsExceedPdoLength = 0x06040042,
    GeneralParameterIncompatibilityReason = 0x06040043,
    GeneralInternalIncompatibilityInDevice = 0x06040047,
    AccessFailedDueToHardwareError = 0x06060000,
    DataTypeDoesNotMatchLengthOfServiceParameter = 0x06070010,
    DataTypeDoesNotMatchLengthTooHigh = 0x06070012,
    DataTypeDoesNotMatchLengthTooLow = 0x06070013,
    SubIndexDoesNotExist = 0x06090011,
    ValueRangeOfParameterExceeded = 0x06090030,
    ValueOfParameterWrittenTooHigh = 0x06090031,
    ValueOfParameterWrittenTooLow = 0x06090032,
    MaximumValueLessThanMinimumValue = 0x06090036,
    GeneralError = 0x08000000,
    DataCannotBeTransferredToApplication = 0x08000020,
    DataCannotBeTransferredDueToLocalControl = 0x08000021,
    DataCannotBeTransferredDueToDeviceState = 0x08000022,
    ObjectDictionaryDynamicGenerationFails = 0x08000023,
}

impl AbortCode {
    /// Returns a description of the abort code.
    pub fn to_str(&self) -> &'static str {
        match self {
            AbortCode::ToggleBitNotAlternated => "Toggle bit not alternated.",
            AbortCode::SdoProtocolTimedOut => "SDO protocol timed out.",
            AbortCode::ClientServerCommandSpecifierInvalid => "Client/server command specifier not valid or unknown.",
            AbortCode::InvalidBlockSize => "Invalid block size (block mode only).",
            AbortCode::InvalidSequenceNumber => "Invalid sequence number (block mode only).",
            AbortCode::CrcError => "CRC error (block mode only).",
            AbortCode::OutOfMemory => "Out of memory.",
            AbortCode::UnsupportedAccessToObject => "Unsupported access to an object.",
            AbortCode::AttemptToReadWriteOnlyObject => "Attempt to read a write only object.",
            AbortCode::AttemptToWriteReadOnlyObject => "Attempt to write a read only object.",
            AbortCode::ObjectDoesNotExistInObjectDictionary => "Object does not exist in the object dictionary.",
            AbortCode::ObjectCannotBeMappedToPdo => "Object cannot be mapped to the PDO.",
            AbortCode::NumberAndLengthOfObjectsExceedPdoLength => "The number and length of the objects to be mapped would exceed PDO length.",
            AbortCode::GeneralParameterIncompatibilityReason => "General parameter incompatibility reason.",
            AbortCode::GeneralInternalIncompatibilityInDevice => "General internal incompatibility in the device.",
            AbortCode::AccessFailedDueToHardwareError => "Access failed due to a hardware error.",
            AbortCode::DataTypeDoesNotMatchLengthOfServiceParameter => "Data type does not match, length of service parameter does not match.",
            AbortCode::DataTypeDoesNotMatchLengthTooHigh => "Data type does not match, length of service parameter too high.",
            AbortCode::DataTypeDoesNotMatchLengthTooLow => "Data type does not match, length of service parameter too low.",
            AbortCode::SubIndexDoesNotExist => "Sub-index does not exist.",
            AbortCode::ValueRangeOfParameterExceeded => "Value range of parameter exceeded (only for write access).",
            AbortCode::ValueOfParameterWrittenTooHigh => "Value of parameter written too high.",
            AbortCode::ValueOfParameterWrittenTooLow => "Value of parameter written too low.",
            AbortCode::MaximumValueLessThanMinimumValue => "Maximum value is less than minimum value.",
            AbortCode::GeneralError => "General error.",
            AbortCode::DataCannotBeTransferredToApplication => "Data cannot be transferred or stored to the application.",
            AbortCode::DataCannotBeTransferredDueToLocalControl => "Data cannot be transferred or stored to the application because of local control.",
            AbortCode::DataCannotBeTransferredDueToDeviceState => "Data cannot be transferred or stored to the application because of the present device state.",
            AbortCode::ObjectDictionaryDynamicGenerationFails => "Object dictionary dynamic generation fails or no object dictionary is present.",
        }
    }
}

impl fmt::Display for AbortCode {
    /// Returns a description of the abort code.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::sdo_abort::AbortCode;

    #[test]
    fn test_abort_code_description() {
        let abort_code = AbortCode::ToggleBitNotAlternated;
        assert_eq!(abort_code.to_str(), "Toggle bit not alternated.");
    }
}
