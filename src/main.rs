use std::io::BufRead;

use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};

mod distributions;
mod summary;

use summary::{DistributionSummary, Summary};

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

fn poisson(matches: &ArgMatches) -> Result<(), failure::Error> {
    let num_experiments = clap::value_t!(matches, "num_experiments", usize)?;
    let lambda = clap::value_t!(matches, "lambda", f64)?;
    distributions::poisson(lambda)?
        .take(num_experiments)
        .for_each(|v| println!("{}", v));
    Ok(())
}

fn exponential(matches: &ArgMatches) -> Result<(), failure::Error> {
    let num_experiments = clap::value_t!(matches, "num_experiments", usize)?;
    let lambda = clap::value_t!(matches, "lambda", f64)?;
    distributions::exponential(lambda)?
        .take(num_experiments)
        .for_each(|v| println!("{}", v));
    Ok(())
}

fn uniform(matches: &ArgMatches) -> Result<(), failure::Error> {
    let num_experiments = clap::value_t!(matches, "num_experiments", usize)?;
    match matches.value_of("type") {
        Some("continuous") => {
            let lower = clap::value_t!(matches, "lower", f64)?;
            let upper = clap::value_t!(matches, "upper", f64)?;
            distributions::continuous_uniform(lower, upper)
                .take(num_experiments)
                .for_each(|v| println!("{}", v));
        }
        Some("discrete") => {
            let lower = clap::value_t!(matches, "lower", i64)?;
            let upper = clap::value_t!(matches, "upper", i64)?;
            distributions::discrete_uniform(lower, upper)
                .take(num_experiments)
                .for_each(|v| println!("{}", v));
        }
        _ => unreachable!(),
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

fn summarize(_matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let mut summary = DistributionSummary::default();
    match input_method {
        InputMethod::Manual => {
            for value in get_results_from_stdin(&mut std::io::stdin()) {
                summary.observe(value?)?;
            }
        }
        InputMethod::Piped => {
            summary.observe_many(get_values_from_stdin(&mut std::io::stdin())?)?;
        }
    }
    println!("{}", summary);
    Ok(())
}

fn mean(_matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let mean = match input_method {
        InputMethod::Manual => summary::mean_result(get_results_from_stdin(&mut std::io::stdin()))?,
        InputMethod::Piped => summary::mean(get_values_from_stdin(&mut std::io::stdin())?),
    };
    println!("{}", mean);
    Ok(())
}

fn variance(matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let (population_variance, sample_variance) = match input_method {
        InputMethod::Manual => {
            summary::variance_result(get_results_from_stdin(&mut std::io::stdin()))?
        }
        InputMethod::Piped => summary::variance(get_values_from_stdin(&mut std::io::stdin())?),
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
        .about("Sample from distributions and calculate summary statistics from the command line.")
        .set_term_width(0)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("gaussian")
                .about("Sample from a normal distribution 𝓝（μ, σ²）")
                .arg(num_experiments.clone())
                .arg(
                    Arg::with_name("mean")
                        .short("m")
                        .long("mean")
                        .help("The mean of the normal random variable, μ.")
                        .default_value("0.0")
                        .allow_hyphen_values(true)
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
            SubCommand::with_name("poisson")
                .about("Sample from a Poisson distribution Pois(λ)")
                .arg(num_experiments.clone())
                .arg(
                    Arg::with_name("lambda")
                        .short("l")
                        .long("lambda")
                        .help("The mean and variance of the Poisson random variable, λ.")
                        .default_value("1.0")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("exponential")
                .about("Sample from a exponential distribution Exp(λ)")
                .arg(num_experiments.clone())
                .arg(
                    Arg::with_name("lambda")
                        .short("l")
                        .long("lambda")
                        .help("The rate of the exponential random variable, λ.")
                        .default_value("1.0")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("uniform")
                .about("Sample from a uniform distribution Uniform(a, b)")
                .after_help(
                    "A continuous uniform distribution is sampled \
                    over [lower, upper), while a discrete uniform distribution \
                    is sampled over {lower, lower+1, ..., upper}.",
                )
                .arg(num_experiments.clone())
                .arg(
                    Arg::with_name("lower")
                        .short("a")
                        .long("lower")
                        .help("The lower bound of the uniform random variable.")
                        .default_value("0")
                        .allow_hyphen_values(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("upper")
                        .short("b")
                        .long("upper")
                        .help("The upper bound of the uniform random variable.")
                        .default_value("1")
                        .allow_hyphen_values(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .help("Whether to use continous or discrete uniform distribution.")
                        .possible_values(&["continuous", "discrete"])
                        .default_value("continuous"),
                ),
        )
        .subcommand(
            SubCommand::with_name("binomial")
                .about("Sample from a binomial distribution Bin(n, p)")
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
            SubCommand::with_name("summarize")
                .about("Calculate basic summary statistics.")
                .after_help("This reads from stdin. You can terminate stdin with CTRL+D."),
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
        ("poisson", Some(matches)) => poisson(matches),
        ("exponential", Some(matches)) => exponential(matches),
        ("uniform", Some(matches)) => uniform(matches),
        ("binomial", Some(matches)) => binomial(matches),
        ("summarize", Some(matches)) => summarize(matches, input_method),
        ("mean", Some(matches)) => mean(matches, input_method),
        ("variance", Some(matches)) => variance(matches, input_method),
        _ => unreachable!(),
    }
}
