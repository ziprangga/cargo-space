use anyhow::Result;

pub use anyhow::Context;
pub use anyhow::anyhow as error;
pub use anyhow::bail as bail_out;

pub type CargoResult<T, E = anyhow::Error> = Result<T, E>;
pub type Error = anyhow::Error;
