mod config;
mod current;
mod deactivate;
mod default;
mod help;
mod install;
mod shim;
mod uninstall;
mod use_;
mod version;

pub(crate) use self::config::Config;
pub(crate) use self::current::Current;
pub(crate) use self::deactivate::Deactivate;
pub(crate) use self::default::Default;
pub(crate) use self::help::Help;
pub(crate) use self::install::Install;
pub(crate) use self::shim::Shim;
pub(crate) use self::uninstall::Uninstall;
pub(crate) use self::use_::Use;
pub(crate) use self::version::Version;

use docopt::Docopt;
use serde::de::DeserializeOwned;

use notion_core::session::Session;
use notion_fail::{FailExt, Fallible};

use {CliParseError, DocoptExt, Notion};

use std::fmt::{self, Display};
use std::str::FromStr;

/// Represents the set of Notion command names.
#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) enum CommandName {
    Install,
    Uninstall,
    Use,
    Config,
    Current,
    Deactivate,
    Default,
    Shim,
    Help,
    Version,
}

impl Display for CommandName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match *self {
                CommandName::Install => "install",
                CommandName::Uninstall => "uninstall",
                CommandName::Use => "use",
                CommandName::Config => "config",
                CommandName::Deactivate => "deactivate",
                CommandName::Default => "default",
                CommandName::Current => "current",
                CommandName::Shim => "shim",
                CommandName::Help => "help",
                CommandName::Version => "version",
            }
        )
    }
}

impl FromStr for CommandName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "install" => CommandName::Install,
            "uninstall" => CommandName::Uninstall,
            "use" => CommandName::Use,
            "config" => CommandName::Config,
            "current" => CommandName::Current,
            "deactivate" => CommandName::Deactivate,
            "default" => CommandName::Default,
            "shim" => CommandName::Shim,
            "help" => CommandName::Help,
            "version" => CommandName::Version,
            _ => {
                throw!(());
            }
        })
    }
}

/// A Notion command.
pub(crate) trait Command: Sized {
    /// The intermediate type Docopt should deserialize the parsed command into.
    type Args: DeserializeOwned;

    /// The full usage documentation for this command. This can contain leading and trailing
    /// whitespace, which will be trimmed before printing to the console.
    const USAGE: &'static str;

    /// Produces a variant of this type representing the `notion <command> --help`
    /// option.
    fn help() -> Self;

    /// Parses the intermediate deserialized arguments into the full command.
    fn parse(notion: Notion, args: Self::Args) -> Fallible<Self>;

    /// Executes the command. Returns `Ok(true)` if the process should return 0,
    /// `Ok(false)` if the process should return 1, and `Err(e)` if the process
    /// should return `e.exit_code()`.
    fn run(self, session: &mut Session) -> Fallible<bool>;

    /// Top-level convenience method for taking a Notion invocation and executing
    /// this command with the arguments taken from the Notion invocation.
    fn go(notion: Notion, session: &mut Session) -> Fallible<bool> {
        let argv = notion.full_argv();
        let args = Docopt::new(Self::USAGE).and_then(|d| d.argv(argv).deserialize());

        match args {
            Ok(args) => Self::parse(notion, args)?.run(session),
            Err(err) => {
                // Docopt models `-h` and `--help` as errors, so this
                // normalizes them to a normal `notion help` command.
                if err.is_help() {
                    Self::help().run(session)
                }
                // Otherwise it's a true docopt error, so rethrow it.
                else {
                    throw!(err.with_context(CliParseError::from_docopt));
                }
            }
        }
    }
}
