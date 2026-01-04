//! # xHCI Types and Constants

// =============================================================================
// TRB TYPES - COMMAND
// =============================================================================

pub const TRB_TYPE_NORMAL: u8 = 1;
pub const TRB_TYPE_SETUP: u8 = 2;
pub const TRB_TYPE_DATA: u8 = 3;
pub const TRB_TYPE_STATUS: u8 = 4;
pub const TRB_TYPE_ISOCH: u8 = 5;
pub const TRB_TYPE_LINK: u8 = 6;
pub const TRB_TYPE_EVENT_DATA: u8 = 7;
pub const TRB_TYPE_NOOP: u8 = 8;

// Command TRBs
pub const TRB_TYPE_ENABLE_SLOT: u8 = 9;
pub const TRB_TYPE_DISABLE_SLOT: u8 = 10;
pub const TRB_TYPE_ADDRESS_DEVICE: u8 = 11;
pub const TRB_TYPE_CONFIGURE_ENDPOINT: u8 = 12;
pub const TRB_TYPE_EVALUATE_CONTEXT: u8 = 13;
pub const TRB_TYPE_RESET_ENDPOINT: u8 = 14;
pub const TRB_TYPE_STOP_ENDPOINT: u8 = 15;
pub const TRB_TYPE_SET_TR_DEQUEUE: u8 = 16;
pub const TRB_TYPE_RESET_DEVICE: u8 = 17;
pub const TRB_TYPE_FORCE_EVENT: u8 = 18;
pub const TRB_TYPE_NEGOTIATE_BANDWIDTH: u8 = 19;
pub const TRB_TYPE_SET_LATENCY: u8 = 20;
pub const TRB_TYPE_GET_PORT_BANDWIDTH: u8 = 21;
pub const TRB_TYPE_FORCE_HEADER: u8 = 22;
pub const TRB_TYPE_NOOP_CMD: u8 = 23;

// Event TRBs
pub const TRB_TYPE_TRANSFER_EVENT: u8 = 32;
pub const TRB_TYPE_COMMAND_COMPLETION: u8 = 33;
pub const TRB_TYPE_PORT_STATUS_CHANGE: u8 = 34;
pub const TRB_TYPE_BANDWIDTH_REQUEST: u8 = 35;
pub const TRB_TYPE_DOORBELL_EVENT: u8 = 36;
pub const TRB_TYPE_HOST_CONTROLLER_EVENT: u8 = 37;
pub const TRB_TYPE_DEVICE_NOTIFICATION: u8 = 38;
pub const TRB_TYPE_MFINDEX_WRAP: u8 = 39;

// =============================================================================
// COMPLETION CODES
// =============================================================================

pub const CC_INVALID: u8 = 0;
pub const CC_SUCCESS: u8 = 1;
pub const CC_DATA_BUFFER_ERROR: u8 = 2;
pub const CC_BABBLE_DETECTED: u8 = 3;
pub const CC_USB_TRANSACTION_ERROR: u8 = 4;
pub const CC_TRB_ERROR: u8 = 5;
pub const CC_STALL_ERROR: u8 = 6;
pub const CC_RESOURCE_ERROR: u8 = 7;
pub const CC_BANDWIDTH_ERROR: u8 = 8;
pub const CC_NO_SLOTS_AVAILABLE: u8 = 9;
pub const CC_INVALID_STREAM_TYPE: u8 = 10;
pub const CC_SLOT_NOT_ENABLED: u8 = 11;
pub const CC_ENDPOINT_NOT_ENABLED: u8 = 12;
pub const CC_SHORT_PACKET: u8 = 13;
pub const CC_RING_UNDERRUN: u8 = 14;
pub const CC_RING_OVERRUN: u8 = 15;
pub const CC_VF_EVENT_RING_FULL: u8 = 16;
pub const CC_PARAMETER_ERROR: u8 = 17;
pub const CC_BANDWIDTH_OVERRUN: u8 = 18;
pub const CC_CONTEXT_STATE_ERROR: u8 = 19;
pub const CC_NO_PING_RESPONSE: u8 = 20;
pub const CC_EVENT_RING_FULL: u8 = 21;
pub const CC_INCOMPATIBLE_DEVICE: u8 = 22;
pub const CC_MISSED_SERVICE: u8 = 23;
pub const CC_COMMAND_RING_STOPPED: u8 = 24;
pub const CC_COMMAND_ABORTED: u8 = 25;
pub const CC_STOPPED: u8 = 26;
pub const CC_STOPPED_LENGTH_INVALID: u8 = 27;
pub const CC_STOPPED_SHORT_PACKET: u8 = 28;
pub const CC_MAX_EXIT_LATENCY_LARGE: u8 = 29;
pub const CC_ISOCH_BUFFER_OVERRUN: u8 = 31;
pub const CC_EVENT_LOST: u8 = 32;
pub const CC_UNDEFINED_ERROR: u8 = 33;
pub const CC_INVALID_STREAM_ID: u8 = 34;
pub const CC_SECONDARY_BANDWIDTH: u8 = 35;
pub const CC_SPLIT_TRANSACTION: u8 = 36;

// =============================================================================
// ENDPOINT TYPES
// =============================================================================

pub const EP_TYPE_ISOCH_OUT: u8 = 1;
pub const EP_TYPE_BULK_OUT: u8 = 2;
pub const EP_TYPE_INTERRUPT_OUT: u8 = 3;
pub const EP_TYPE_CONTROL: u8 = 4;
pub const EP_TYPE_ISOCH_IN: u8 = 5;
pub const EP_TYPE_BULK_IN: u8 = 6;
pub const EP_TYPE_INTERRUPT_IN: u8 = 7;

// =============================================================================
// SLOT/ENDPOINT STATE
// =============================================================================

pub const SLOT_STATE_DISABLED: u8 = 0;
pub const SLOT_STATE_DEFAULT: u8 = 1;
pub const SLOT_STATE_ADDRESSED: u8 = 2;
pub const SLOT_STATE_CONFIGURED: u8 = 3;

pub const EP_STATE_DISABLED: u8 = 0;
pub const EP_STATE_RUNNING: u8 = 1;
pub const EP_STATE_HALTED: u8 = 2;
pub const EP_STATE_STOPPED: u8 = 3;
pub const EP_STATE_ERROR: u8 = 4;

// =============================================================================
// PORT SPEEDS
// =============================================================================

pub const SPEED_FULL: u8 = 1;
pub const SPEED_LOW: u8 = 2;
pub const SPEED_HIGH: u8 = 3;
pub const SPEED_SUPER: u8 = 4;
pub const SPEED_SUPER_PLUS: u8 = 5;

// =============================================================================
// MISC CONSTANTS
// =============================================================================

/// Número máximo de slots suportados.
pub const MAX_SLOTS: usize = 256;

/// Número máximo de endpoints por dispositivo (32 = 16 in + 16 out).
pub const MAX_ENDPOINTS: usize = 32;

/// Tamanho de um TRB em bytes.
pub const TRB_SIZE: usize = 16;

/// Número de TRBs em um ring por padrão.
pub const DEFAULT_RING_SIZE: usize = 256;

/// Doorbell value para command ring.
pub const DOORBELL_HOST_CONTROLLER: u8 = 0;
