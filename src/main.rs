use std::io::BufRead;

use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};

mod aggregates;
mod distributions;

enum InputMethod {
    Manual,
    Piped,
}

fn gaussian(matches: &ArgMatches) -> Result<(), failure::Error> {
    let num_experiments = clap::value_t!(matches, "num_experiments", usize)?;
    let mean = clap::value_t!(matches, "mean", f64)?;
    let variance = clap::value_t!(matches, "variance", f64)?;
    if mean == 0.0 && variance == 1.0 {
        distributions::standard_gaussian()
            .take(num_experiments)
            .for_each(|v| println!("{}", v));
    } else {
        distributions::gaussian(mean, variance)?
            .take(num_experiments)
            .for_each(|v| println!("{}", v));
    };
    Ok(())
}

fn binomial(matches: &ArgMatches) -> Result<(), failure::Error> {
    let num_experiments = clap::value_t!(matches, "num_experiments", usize)?;
    let num_trials = clap::value_t!(matches, "num_trials", u64)?;
    let probability = clap::value_t!(matches, "probability", f64)?;
    distributions::binomial(num_trials, probability)?
        .take(num_experiments)
        .for_each(|v| println!("{}", v));
    Ok(())
}

fn mean(_matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let mean = match input_method {
        InputMethod::Manual => {
            aggregates::mean_result(get_results_from_stdin(&mut std::io::stdin()))?
        }
        InputMethod::Piped => aggregates::mean(get_values_from_stdin(&mut std::io::stdin())?),
    };
    println!("{}", mean);
    Ok(())
}

fn variance(matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let (population_variance, sample_variance) = match input_method {
        InputMethod::Manual => {
            aggregates::variance_result(get_results_from_stdin(&mut std::io::stdin()))?
        }
        InputMethod::Piped => aggregates::variance(get_values_from_stdin(&mut std::io::stdin())?),
    };
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
) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let results = get_results_from_stdin(stdin);
    results
        .collect::<Result<Vec<f64>, failure::Error>>()
        .map(|i| i.into_iter())
}

fn get_results_from_stdin(
    stdin: &mut std::io::Stdin,
) -> impl Iterator<Item = Result<f64, failure::Error>> + '_ {
    stdin.lock().lines().map(|line| Ok(line?.parse::<f64>()?))
}

fn main() -> Result<(), failure::Error> {
    let num_experiments = Arg::with_name("num_experiments")
        .short("N")
        .long("num_experiments")
        .help("The number of experiments to perform.")
        .default_value("1")
        .takes_value(true);

    let app_matches = App::new("samplers")
        .author("mingyli")
        .about("Sample from distributions and calculate aggregates from the command line.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("gaussian")
                .about("Sample from a normal distribution.")
                .arg(num_experiments.clone())
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
            SubCommand::with_name("binomial")
                .about("Sample from a binomial distribution.")
                .arg(num_experiments.clone())
                .arg(
                    Arg::with_name("num_trials")
                        .short("n")
                        .long("num_trials")
                        .help("The number of independent trials to perform.")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("probability")
                        .short("p")
                        .long("probability")
                        .help("The probability of success for each trial.")
                        .default_value("0.5")
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

    let input_method = if atty::is(atty::Stream::Stdin) {
        InputMethod::Manual
    } else {
        InputMethod::Piped
    };

    match app_matches.subcommand() {
        ("gaussian", Some(matches)) => gaussian(matches),
        ("binomial", Some(matches)) => binomial(matches),
        ("mean", Some(matches)) => mean(matches, input_method),
        ("variance", Some(matches)) => variance(matches, input_method),
        _ => unreachable!(),
    }
}
