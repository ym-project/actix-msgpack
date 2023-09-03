// Why is 256? Because actix-web has the same limit by default.

/// Payload max size (256Kb)
pub const DEFAULT_PAYLOAD_LIMIT: usize = 262_144;
