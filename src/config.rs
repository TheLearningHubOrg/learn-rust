//! Stage 1: turn an environment variable into a typed, validated value.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use thiserror::Error;

/// Port used when `PORT` is unset or empty. Must match `EXPOSE` in the Dockerfile.
pub const DEFAULT_PORT: u16 = 8080;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub port: u16,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("PORT must be a whole number from 1 to 65535, got {0:?}")]
    InvalidPort(String),
}

impl Config {
    /// Builds a config from the raw value of `PORT`, if there is one.
    ///
    /// Taking `Option<&str>` instead of reading the environment keeps this
    /// function pure, so tests never have to mutate process-wide state.
    pub fn from_port_var(raw: Option<&str>) -> Result<Self, ConfigError> {
        let port = match raw.map(str::trim) {
            None | Some("") => DEFAULT_PORT,
            Some(text) => match text.parse::<u16>() {
                Ok(0) | Err(_) => return Err(ConfigError::InvalidPort(text.to_string())),
                Ok(port) => port,
            },
        };
        Ok(Self { port })
    }

    /// Reads `PORT` from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        let raw = std::env::var("PORT").ok();
        Self::from_port_var(raw.as_deref())
    }

    /// Binds every interface (0.0.0.0) so the server is reachable inside a container.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_blank_port_falls_back_to_default() {
        assert_eq!(Config::from_port_var(None).unwrap().port, DEFAULT_PORT);
        assert_eq!(
            Config::from_port_var(Some("  ")).unwrap().port,
            DEFAULT_PORT
        );
    }

    #[test]
    fn valid_port_is_parsed_and_trimmed() {
        assert_eq!(Config::from_port_var(Some(" 3000 ")).unwrap().port, 3000);
    }

    #[test]
    fn out_of_range_zero_and_text_are_rejected() {
        for bad in ["0", "65536", "-1", "eighty"] {
            assert_eq!(
                Config::from_port_var(Some(bad)),
                Err(ConfigError::InvalidPort(bad.to_string())),
                "input {bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn socket_addr_binds_all_interfaces() {
        let addr = Config { port: 9000 }.socket_addr();
        assert_eq!(addr.to_string(), "0.0.0.0:9000");
    }
}
