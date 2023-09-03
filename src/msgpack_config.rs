use super::DEFAULT_PAYLOAD_LIMIT;

pub struct MsgPackConfig {
	pub(crate) limit: usize,
}

impl MsgPackConfig {
	/// Set maximum accepted payload size in bytes. The default limit is 256KiB.
	pub fn limit(&mut self, limit: usize) -> &mut Self {
		self.limit = limit;
		self
	}
}

impl Default for MsgPackConfig {
	fn default() -> Self {
		MsgPackConfig { limit: DEFAULT_PAYLOAD_LIMIT }
	}
}
