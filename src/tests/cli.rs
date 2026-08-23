use super::Cli;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parse_should_accept_long_and_attached_queries() -> anyhow::Result<()> {
    assert_eq!(
        Cli::parse(args(&["--query=10 miles to km"]))?.query,
        "10 miles to km"
    );
    Ok(())
}

#[test]
fn parse_should_accept_collapsed_flags_and_query_cluster() -> anyhow::Result<()> {
    let cli = Cli::parse(args(&["-CvUq10 mi km"]))?;
    assert_eq!(
        cli,
        Cli {
            query: "10 mi km".to_owned(),
            currencies: true,
            units: true,
            verbose: true,
            update: false,
        }
    );
    Ok(())
}

#[test]
fn parse_should_reject_cluster_as_separated_query_value() {
    let error = Cli::parse(args(&["--query", "-vU"]));
    assert_eq!(
        error.map_err(|error| error.to_string()),
        Err("--query requires a value".to_owned())
    );
}

#[test]
fn parse_should_retain_unrecognized_dash_prefixed_query() -> anyhow::Result<()> {
    assert_eq!(
        Cli::parse(args(&["-q", "-10 m to cm"]))?.query,
        "-10 m to cm"
    );
    Ok(())
}

#[test]
fn normalized_query_should_collapse_whitespace_without_lowercasing() {
    let cli = Cli {
        query: "  10\tMi   KM ".to_owned(),
        ..Cli::default()
    };
    assert_eq!(cli.normalized_query(), "10 Mi KM");
}
