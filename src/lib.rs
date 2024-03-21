//! `command_error` provides the [`CommandExt`] trait, which runs a command and checks its exit
//! status:
//!
//! ```
//! # use indoc::indoc;
//! use std::process::Command;
//! use command_error::CommandExt;
//!
//! let err = Command::new("sh")
//!     .args(["-c", "echo puppy; false"])
//!     .output_checked_utf8()
//!     .unwrap_err();
//!
//! assert_eq!(
//!     err.to_string(),
//!     indoc!(
//!         "`sh` failed: exit status: 1
//!         Command failed: `sh -c 'echo puppy; false'`
//!         Stdout:
//!           puppy"
//!     )
//! );
//! ```
//!
//! Error messages are detailed and helpful. Additional methods are provided for overriding
//! the default success logic (for that weird tool that thinks `2` is a reasonable exit code) and
//! for transforming the output (for example, to parse command output as JSON while retaining
//! information about the command that produced the output in the error message).
//!
//! ## Enforcing use of `command_error`
//!
//! If you'd like to make sure that [`CommandExt`] methods are used instead of the plain
//! [`Command`] methods in your project, you can add a stanza like this to
//! [`clippy.toml`][clippy-config] at your project root:
//!
//! ```toml
//! [[disallowed-methods]]
//! path = "std::process::Command::output"
//! reason = "Use command_error::CommandExt::output_checked[_with][_utf8]"
//!
//! [[disallowed-methods]]
//! path = "std::process::Command::status"
//! reason = "Use command_error::CommandExt::status_checked[_with]"
//! ```
//!
//! [clippy-config]: https://doc.rust-lang.org/clippy/configuration.html

mod output_context;
pub use output_context::OutputContext;

mod output_like;
pub use output_like::OutputLike;

mod exec_error;
pub use exec_error::ExecError;

mod output_error;
pub use output_error::OutputError;

mod output_conversion_error;
pub use output_conversion_error::OutputConversionError;

mod error;
pub use error::Error;

mod command_display;
pub use command_display::CommandDisplay;
pub use command_display::Utf8ProgramAndArgs;

mod debug_display;
pub(crate) use debug_display::DebugDisplay;

mod command_ext;
pub use command_ext::CommandExt;
