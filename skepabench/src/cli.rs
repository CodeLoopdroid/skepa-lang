use std::path::PathBuf;

use crate::{CliOptions, DEFAULT_RUNS, DEFAULT_WARMUPS};

pub(crate) fn parse_args(mut args: impl Iterator<Item = String>) -> Result<CliOptions, String> {
    let mut warmups = DEFAULT_WARMUPS;
    let mut runs = DEFAULT_RUNS;
    let mut profile = String::from("debug");
    let mut filter = None;
    let mut json = false;
    let mut save_baseline = false;
    let mut compare = false;
    let mut baseline_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--warmups" => {
                let Some(value) = args.next() else {
                    return Err("Missing value for --warmups".to_string());
                };
                warmups = value
                    .parse::<usize>()
                    .map_err(|_| "--warmups must be a positive integer".to_string())?;
            }
            "--runs" => {
                let Some(value) = args.next() else {
                    return Err("Missing value for --runs".to_string());
                };
                runs = value
                    .parse::<usize>()
                    .map_err(|_| "--runs must be a positive integer".to_string())?;
            }
            "--profile" => {
                let Some(value) = args.next() else {
                    return Err("Missing value for --profile".to_string());
                };
                if value != "debug" && value != "release" {
                    return Err("--profile must be `debug` or `release`".to_string());
                }
                profile = value;
            }
            "--filter" => {
                let Some(value) = args.next() else {
                    return Err("Missing value for --filter".to_string());
                };
                filter = Some(value);
            }
            "--json" => json = true,
            "--save-baseline" => save_baseline = true,
            "--compare" => compare = true,
            "--baseline-path" => {
                let Some(value) = args.next() else {
                    return Err("Missing value for --baseline-path".to_string());
                };
                baseline_path = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return Err(
                    "Usage: cargo run -p skepabench -- [--warmups N] [--runs N] [--profile debug|release] [--filter SUBSTR] [--json] [--save-baseline] [--compare] [--baseline-path PATH]"
                        .to_string(),
                );
            }
            _ => return Err(format!("Unknown argument `{arg}`")),
        }
    }

    if warmups == 0 || runs == 0 {
        return Err("--warmups and --runs must be >= 1".to_string());
    }

    Ok(CliOptions {
        warmups,
        runs,
        profile,
        filter,
        json,
        save_baseline,
        compare,
        baseline_path,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn parse_args_accepts_valid_combinations() {
        let opts = parse_args(
            [
                "--warmups",
                "2",
                "--runs",
                "3",
                "--profile",
                "release",
                "--filter",
                "runtime",
                "--json",
                "--save-baseline",
                "--compare",
                "--baseline-path",
                "x.tsv",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect("args should parse");

        assert_eq!(opts.warmups, 2);
        assert_eq!(opts.runs, 3);
        assert_eq!(opts.profile, "release");
        assert_eq!(opts.filter.as_deref(), Some("runtime"));
        assert!(opts.json);
        assert!(opts.save_baseline);
        assert!(opts.compare);
        assert_eq!(
            opts.baseline_path.as_deref(),
            Some(std::path::Path::new("x.tsv"))
        );
    }

    #[test]
    fn parse_args_rejects_zero_and_unknown_values() {
        assert!(parse_args(["--runs", "0"].into_iter().map(str::to_string)).is_err());
        assert!(parse_args(["--profile", "fast"].into_iter().map(str::to_string)).is_err());
        assert!(parse_args(["--wat"].into_iter().map(str::to_string)).is_err());
    }
}
