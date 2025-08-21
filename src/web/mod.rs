#[cfg(feature = "web")]
pub mod api;
#[cfg(feature = "web")]
pub mod handlers;
#[cfg(feature = "web")]
pub mod server;

#[cfg(feature = "web")]
pub use server::start_web_server;