use crate::ics02_client::client_type::ClientType;
use crate::Height;

/// Abstract of consensus state misbehaviour information
#[dyn_clonable::clonable]
pub trait Misbehaviour: Clone + std::fmt::Debug + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// The height of misbehaviour
    fn height(&self) -> Height;

    /// Performs basic validation of misbehaviour
    fn validate_basic(&self) -> Result<(), Box<dyn std::error::Error>>;
}
