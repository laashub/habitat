#[cfg(windows)]
pub mod windows_child;

#[allow(unused_variables)]
#[cfg(windows)]
mod windows;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub(crate) use self::unix::SignalCode;
#[cfg(unix)]
pub use self::unix::{become_command,
                     current_pid,
                     is_alive,
                     signal,
                     Pid};
#[cfg(windows)]
pub use self::windows::{become_command,
                        current_pid,
                        handle_from_pid,
                        is_alive,
                        terminate,
                        Pid};
use crate::{error::Error,
            util::serde_string};
use serde_derive::{Deserialize,
                   Serialize};
use std::{fmt,
          result,
          str::FromStr,
          time::Duration};

/// This type encapsulates the number of seconds we should wait after
/// send a shutdown signal to a process before we kill it.
#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Copy, Hash)]
#[serde(from = "u32")]
pub struct ShutdownTimeout(u32);

impl Default for ShutdownTimeout {
    /// Unless otherwise specified, the Supervisor will wait 8 seconds
    /// for a service to finish shutting down before killing it.
    fn default() -> Self { 8.into() }
}

impl FromStr for ShutdownTimeout {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(ShutdownTimeout(s.parse()?)) }
}

impl fmt::Display for ShutdownTimeout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<u32> for ShutdownTimeout {
    fn from(seconds: u32) -> Self { ShutdownTimeout(seconds) }
}

impl From<ShutdownTimeout> for u32 {
    fn from(timeout: ShutdownTimeout) -> Self { timeout.0 }
}

impl From<ShutdownTimeout> for Duration {
    fn from(timeout: ShutdownTimeout) -> Self { Duration::from_secs(timeout.0.into()) }
}

// This defines a handful of Unix signals that we want to deal with,
// but we are making it available on Windows as well for situations
// where a Windows CLI is communicating with a Linux Supervisor.
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
pub enum Signal {
    INT,
    ILL,
    ABRT,
    FPE,
    KILL,
    SEGV,
    TERM,
    HUP,
    QUIT,
    ALRM,
    USR1,
    USR2,
    CHLD,
}

impl FromStr for Signal {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let signal = match s {
            "HUP" => Signal::HUP,
            "INT" => Signal::INT,
            "QUIT" => Signal::QUIT,
            "ILL" => Signal::ILL,
            "ABRT" => Signal::ABRT,
            "FPE" => Signal::FPE,
            "KILL" => Signal::KILL,
            "USR1" => Signal::USR1,
            "SEGV" => Signal::SEGV,
            "USR2" => Signal::USR2,
            "ALRM" => Signal::ALRM,
            "TERM" => Signal::TERM,
            "CHLD" => Signal::CHLD,
            _ => return Err(Error::ParseSignalError(s.to_string())),
        };
        Ok(signal)
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Signal::HUP => "HUP",
            Signal::INT => "INT",
            Signal::QUIT => "QUIT",
            Signal::ILL => "ILL",
            Signal::ABRT => "ABRT",
            Signal::FPE => "FPE",
            Signal::KILL => "KILL",
            Signal::USR1 => "USR1",
            Signal::SEGV => "SEGV",
            Signal::USR2 => "USR2",
            Signal::ALRM => "ALRM",
            Signal::TERM => "TERM",
            Signal::CHLD => "CHLD",
        };
        write!(f, "{}", s)
    }
}

/// Encapsulates logic for defining the default shutdown signal we
/// send services, and handles translation from external types at the
/// edges of our system.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct ShutdownSignal(#[serde(with = "serde_string")] Signal);

impl Default for ShutdownSignal {
    /// Unless otherwise specified, the Supervisor will shut down
    /// services by sending the `TERM` signal.
    fn default() -> Self { Signal::TERM.into() }
}

impl FromStr for ShutdownSignal {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> { Ok(ShutdownSignal(s.parse()?)) }
}

impl fmt::Display for ShutdownSignal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<Signal> for ShutdownSignal {
    fn from(signal: Signal) -> Self { ShutdownSignal(signal) }
}

impl From<ShutdownSignal> for Signal {
    fn from(shutdown_signal: ShutdownSignal) -> Self { shutdown_signal.0 }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn signal_names_are_only_accepted_as_uppercase() {
        assert_eq!(Signal::HUP, "HUP".parse().unwrap());
        assert!("hup".parse::<Signal>().is_err());
    }

    #[test]
    fn signals_can_render_as_strings() {
        assert_eq!("HUP", Signal::HUP.to_string());
    }

    #[test]
    fn signals_can_round_trip_through_parsing() {
        for signal in &[Signal::HUP,
                        Signal::INT,
                        Signal::QUIT,
                        Signal::ABRT,
                        Signal::FPE,
                        Signal::KILL,
                        Signal::USR1,
                        Signal::SEGV,
                        Signal::USR2,
                        Signal::ALRM,
                        Signal::TERM,
                        Signal::CHLD]
        {
            assert_eq!(*signal,
                       signal.to_string()
                             .parse::<Signal>()
                             .expect("Couldn't parse back into a Signal!"));
        }
    }
}
