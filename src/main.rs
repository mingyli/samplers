use std::io::BufRead;

use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};

mod aggregates;
mod distributions;

fn gaussian(matches: &ArgMatches) -> Result<(), failure::Error> {
    let n = clap::value_t!(matches, "num_samples", usize)?;
    let mean = clap::value_t!(matches, "mean", f64)?;
    let variance = clap::value_t!(matches, "variance", f64)?;
    if mean == 0.0 && variance == 1.0 {
        distributions::standard_gaussian()
            .take(n)
            .for_each(|v| println!("{}", v));
    } else {
        distributions::gaussian(mean, variance)?
            .take(n)
            .for_each(|v| println!("{}", v));
    };
    Ok(())
}

fn mean(_matches: &ArgMatches) -> Result<(), failure::Error> {
    println!(
        "{}",
        aggregates::mean(get_values_from_stdin(&mut std::io::stdin()))?
    );
    Ok(())
}

fn variance(matches: &ArgMatches) -> Result<(), failure::Error> {
    let (population_variance, sample_variance) =
        aggregates::variance(get_values_from_stdin(&mut std::io::stdin()))?;
    println!(
        "{}",
        match matches.value_of("type") {
            Some("population") => population_variance,
            Some("sample") => sample_variance,
            _ => unreachable!(),
        }
    );
    Ok(())
}

fn get_values_from_stdin(
    stdin: &mut std::io::Stdin,
) -> impl Iterator<Item = Result<f64, failure::Error>> + '_ {
    stdin.lock().lines().map(|line| Ok(line?.parse::<f64>()?))
}

fn main() -> Result<(), failure::Error> {
    let app_matches = App::new("samplers")
        .author("mingyli")
        .about("Sample from distributions and calculate aggregates from the command line.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("gaussian")
                .about("Sample from a normal distribution.")
                .arg(
                    Arg::with_name("num_samples")
                        .short("n")
                        .long("num_samples")
                        .help("The number of samples to take.")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mean")
                        .short("m")
                        .long("mean")
                        .help("The mean of the normal random variable, μ.")
                        .default_value("0.0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("variance")
                        .short("v")
                        .long("variance")
                        .help("The variance of the normal random variable, σ².")
                        .default_value("1.0")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("mean")
                .about("Calculate the mean of given values.")
                .after_help("This reads from stdin. You can terminate stdin with CTRL+D."),
        )
        .subcommand(
            SubCommand::with_name("variance")
                .about("Calculate the variance of given values.")
                .after_help("This reads from stdin. You can terminate stdin with CTRL+D.")
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .help("Whether to compute population variance or sample variance.")
                        .possible_values(&["population", "sample"])
                        .default_value("population"),
                ),
        )
        .get_matches();

    match app_matches.subcommand() {
        ("gaussian", Some(matches)) => gaussian(matches),
        ("mean", Some(matches)) => mean(matches),
        ("variance", Some(matches)) => variance(matches),
        _ => unreachable!(),
    }
}
