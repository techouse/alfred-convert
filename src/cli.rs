//! Command-line parsing for Alfred's Script Filter invocation.

use anyhow::{Result, bail};

/// Command-line options accepted by the workflow executable.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Conversion query supplied by Alfred.
    pub query: String,
    /// Lists supported currencies.
    pub currencies: bool,
    /// Lists supported physical units.
    pub units: bool,
    /// Enables diagnostic output on stderr.
    pub verbose: bool,
    /// Downloads and opens a newer workflow release.
    pub update: bool,
}

impl Cli {
    /// Parses command-line arguments without the executable name.
    ///
    /// # Errors
    /// Returns an error for unknown options or a missing query value.
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut cli = Self::default();
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "-q" | "--query" => {
                    cli.query = take_query_value(&argument, &mut arguments)?;
                }
                "-C" | "--currencies" => cli.currencies = true,
                "-U" | "--units" => cli.units = true,
                "-v" | "--verbose" => cli.verbose = true,
                "-u" | "--update" => cli.update = true,
                _ if argument.starts_with("--query=") => {
                    argument["--query=".len()..].clone_into(&mut cli.query);
                }
                _ if is_valid_short_option_cluster(&argument) => {
                    parse_short_cluster(&argument, &mut arguments, &mut cli)?;
                }
                _ => bail!("unknown argument: {argument}"),
            }
        }

        Ok(cli)
    }

    /// Collapses whitespace while preserving the query's letter case.
    #[must_use]
    pub fn normalized_query(&self) -> String {
        self.query.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn parse_short_cluster(
    argument: &str,
    arguments: &mut impl Iterator<Item = String>,
    cli: &mut Cli,
) -> Result<()> {
    for (offset, character) in argument[1..].char_indices() {
        match character {
            'C' => cli.currencies = true,
            'U' => cli.units = true,
            'v' => cli.verbose = true,
            'u' => cli.update = true,
            'q' => {
                let attached = &argument[1 + offset + 1..];
                cli.query = if attached.is_empty() {
                    take_query_value("-q", arguments)?
                } else {
                    attached.to_owned()
                };
                break;
            }
            _ => bail!("unknown argument: {argument}"),
        }
    }
    Ok(())
}

fn take_query_value(option: &str, arguments: &mut impl Iterator<Item = String>) -> Result<String> {
    let value = arguments
        .next()
        .ok_or_else(|| anyhow::anyhow!("{option} requires a value"))?;
    if matches!(
        value.as_str(),
        "-q" | "--query"
            | "-C"
            | "--currencies"
            | "-U"
            | "--units"
            | "-v"
            | "--verbose"
            | "-u"
            | "--update"
    ) || value.starts_with("--query=")
        || is_valid_short_option_cluster(&value)
    {
        bail!("{option} requires a value");
    }

    Ok(value)
}

fn is_valid_short_option_cluster(argument: &str) -> bool {
    if argument.len() <= 2 || !argument.starts_with('-') || argument.starts_with("--") {
        return false;
    }

    for character in argument[1..].chars() {
        match character {
            'C' | 'U' | 'v' | 'u' => {}
            'q' => return true,
            _ => return false,
        }
    }

    true
}

#[cfg(test)]
#[path = "tests/cli.rs"]
mod tests;
