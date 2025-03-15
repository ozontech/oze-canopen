use binrw::binrw;

/// Represents the common EMCY error codes.
///
/// The EMCY (Emergency) object is triggered when an error is detected by the device.
/// It is suitable for error notifications similar to interrupts. The EMCY object is
/// transmitted only once per error event. No further EMCY objects should be sent until
/// new device errors occur. The EMCY object may not be received by anyone, or it may
/// be received by one or more consumers. The reaction of the EMCY consumer(s) is not
/// specified and is outside the scope of this document. The semantic values of the
/// emergency error codes and the error register are defined, while additional device-
/// specific information and conditions for the emergency are outside the scope of this
/// specification.
///
/// The device can have one of two emergency states. Depending on the transitions
/// between states, emergency objects will be sent. The relationship between the error
/// state machine and the NMT state machine is defined in the device profiles.
///
/// The emergency object is optional. If a device supports the emergency object, it
/// must support at least two error codes: 00xx (Error Reset or No Error) and 10xx
/// (Generic Error). All other error codes are optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw]
#[brw(little, repr=u8)]
pub enum EmcyCode {
    /// Error Reset or No Error
    NoError = 0x00,
    /// Generic Error
    GenericError = 0x10,
    /// Current
    Current = 0x20,
    /// Current - at input
    CurrentInput = 0x21,
    /// Current - inside the device
    CurrentInside = 0x22,
    /// Current - at output
    CurrentOutput = 0x23,
    /// Voltage
    Voltage = 0x30,
    /// Voltage - Mains Voltage
    VoltageMains = 0x31,
    /// Voltage - inside the device
    VoltageInside = 0x32,
    /// Voltage - at output
    VoltageOutput = 0x33,
    /// Temperature
    Temperature = 0x40,
    /// Temperature - Ambient Temperature
    TemperatureAmbient = 0x41,
    /// Temperature - Device Temperature
    TemperatureDevice = 0x42,
    /// Device Hardware
    DeviceHardware = 0x50,
    /// Device Software
    Software = 0x60,
    /// Device Software - Internal Software
    SoftwareInternal = 0x61,
    /// Device Software - User Software
    SoftwareUser = 0x62,
    /// Device Software - Data Set
    SoftwareDataSet = 0x63,
    /// Additional Modules
    AdditionalModules = 0x70,
    /// Monitoring
    Monitoring = 0x80,
    /// Monitoring Communication
    MonitoringCommunication = 0x81,
    /// Monitoring Protocol Error
    MonitoringProtocolError = 0x82,
    /// External Error
    ExternalError = 0x90,
    /// Additional Functions
    AdditionalFunctions = 0xF0,
    /// Device specific
    DeviceSpecific = 0xFF,
}

/// Represents an EMCY message.
///
/// The emergency message consists of 8 bytes of data. The first byte is the
/// Emergency Error Code, followed by the Error Register, and then a
/// manufacturer-specific error field.
#[binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[brw(little)]
pub struct Emcy {
    /// Emergency Error Code
    pub code: EmcyCode,
    /// Custom vendor code
    pub vendor_code: u8,
    /// register number in 0x1001
    pub error_register: u8,
    /// Vendor specific data
    pub data: [u8; 5],
}
