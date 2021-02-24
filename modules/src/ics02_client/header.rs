use crate::ics02_client::client_type::ClientType;
use crate::Height;

use super::client_def::AnyHeader;

/// Abstract of consensus state update information
#[dyn_clonable::clonable]
pub trait Header: Clone + std::fmt::Debug + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// The height of the header
    fn height(&self) -> Height;

    /// Performs basic validation of the header
    fn validate_basic(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Wrap into an `AnyHeader`
    fn wrap_any(self) -> AnyHeader;
}
