#[macro_use]
extern crate failure;

use std::io::BufRead;

use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};

mod distributions;
mod histogram;
mod render;
mod summary;

use histogram::Histogram;
use summary::{DistributionSummary, Observer};

#[derive(Debug, Fail)]
enum SamplersError {
    #[fail(display = "Could not observe value: {}", value)]
    CouldNotObserveValue { value: f64 },
    #[fail(display = "Could not calculate summary statistic: {}", name)]
    CouldNotCalculateSummaryStatistic { name: String },
}

enum InputMethod {
    Manual,
    Piped,
}

#[derive(PartialEq, Debug)]
enum OutputMethod {
    Console,
    Piped,
}

fn gaussian(matches: &ArgMatches) -> Result<(), failure::Error> {
    let num_experiments = clap::value_t!(matches, "num_experiments", usize)?;
    let mean = clap::value_t!(matches, "mean", f64)?;
    let variance = clap::value_t!(matches, "variance", f64)?;
    distributions::gaussian(mean, variance)?
        .take(num_experiments)
        .for_each(|v| println!("{}", v));
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
    let num_trials = clap::value_t!(matches, "num-trials", u64)?;
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
                summary.observe(&value?)?;
            }
        }
        InputMethod::Piped => {
            summary.observe_many(get_values_from_stdin()?.iter())?;
        }
    }
    println!("{}", summary);
    Ok(())
}

fn histogram(matches: &ArgMatches, output_method: OutputMethod) -> Result<(), failure::Error> {
    let num_buckets: usize = clap::value_t!(matches, "num-buckets", usize)?;
    let display_size: usize = clap::value_t!(matches, "display-size", usize)?;
    let histogram = match (
        clap::value_t!(matches, "min", f64),
        clap::value_t!(matches, "max", f64),
    ) {
        (Ok(min), Ok(max)) => {
            // Compute histogram in a single pass.
            let mut histogram = Histogram::with_bounds(min, max, num_buckets);
            get_results_from_stdin(&mut std::io::stdin()).try_for_each(|result| {
                let value = result?;
                if output_method == OutputMethod::Piped {
                    println!("{}", value);
                }
                histogram.observe(&value)
            })?;
            histogram
        }
        (min_result, max_result) => {
            let values: Vec<f64> = get_values_from_stdin()?;
            if output_method == OutputMethod::Piped {
                values.iter().for_each(|value| println!("{}", value));
            }
            let mut summary = DistributionSummary::default();
            summary.observe_many(values.iter())?;
            let min = min_result.or_else(|_| {
                summary
                    .min()
                    .ok_or_else(|| SamplersError::CouldNotCalculateSummaryStatistic {
                        name: "min".to_string(),
                    })
            })?;
            let max = max_result.or_else(|_| {
                summary
                    .max()
                    .ok_or_else(|| SamplersError::CouldNotCalculateSummaryStatistic {
                        name: "max".to_string(),
                    })
            })?;
            let mut histogram = Histogram::with_bounds(min, max, num_buckets);
            histogram.observe_many(values.iter())?;
            histogram
        }
    };
    let buckets = histogram.collect();
    match output_method {
        OutputMethod::Console => render::render_buckets(&buckets, display_size, std::io::stdout()),
        OutputMethod::Piped => render::render_buckets(&buckets, display_size, std::io::stderr()),
    }
}

fn mean(_matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let mean = match input_method {
        InputMethod::Manual => summary::mean_result(get_results_from_stdin(&mut std::io::stdin()))?,
        InputMethod::Piped => summary::mean(get_values_from_stdin()?.into_iter()),
    };
    println!("{}", mean);
    Ok(())
}

fn variance(matches: &ArgMatches, input_method: InputMethod) -> Result<(), failure::Error> {
    let (population_variance, sample_variance) = match input_method {
        InputMethod::Manual => {
            summary::variance_result(get_results_from_stdin(&mut std::io::stdin()))?
        }
        InputMethod::Piped => summary::variance(get_values_from_stdin()?.into_iter()),
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

fn get_values_from_stdin() -> Result<Vec<f64>, failure::Error> {
    let mut stdin = std::io::stdin();
    let results = get_results_from_stdin(&mut stdin);
    results.collect::<Result<Vec<f64>, failure::Error>>()
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
        .about(
            "Sample from common distributions and calculate summary statistics from the command \
             line.",
        )
        .version("0.1.3")
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
                .about("Sample from an exponential distribution Exp(λ)")
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
                    "A continuous uniform distribution is sampled over [lower, upper), while a \
                     discrete uniform distribution is sampled over {lower, lower+1, ..., upper}.",
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
                    Arg::with_name("num-trials")
                        .short("n")
                        .long("num-trials")
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
                .after_help(
                    "This reads from stdin. You can terminate stdin with CTRL+D.\nBy default, \
                     this command computes summary statistics in a single pass with a constant \
                     amount of additional memory.",
                ),
        )
        .subcommand(
            SubCommand::with_name("histogram")
                .about("Displays a histogram of given values.")
                .after_help(
                    "This reads from stdin. You can terminate stdin with CTRL+D.\nIf this output \
                     is being piped, it will duplicate its input to stdout and print the \
                     histogram to stderr instead.\nIf the minimum and maximum bounds of the \
                     histogram are provided ahead of time, the histogram will be computed in a \
                     single pass.",
                )
                .arg(
                    Arg::with_name("min")
                        .long("min")
                        .help("The lowest boundary in the histogram.")
                        .allow_hyphen_values(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("max")
                        .long("max")
                        .help("The highest boundary in the histogram.")
                        .allow_hyphen_values(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("num-buckets")
                        .short("b")
                        .long("num-buckets")
                        .help("The number of buckets in the histogram.")
                        .default_value("15")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("display-size")
                        .short("d")
                        .long("display-size")
                        .help("The size of the histogram in the terminal.")
                        .default_value("80")
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

    let output_method = if atty::is(atty::Stream::Stdout) {
        OutputMethod::Console
    } else {
        OutputMethod::Piped
    };

    match app_matches.subcommand() {
        ("gaussian", Some(matches)) => gaussian(matches),
        ("poisson", Some(matches)) => poisson(matches),
        ("exponential", Some(matches)) => exponential(matches),
        ("uniform", Some(matches)) => uniform(matches),
        ("binomial", Some(matches)) => binomial(matches),
        ("summarize", Some(matches)) => summarize(matches, input_method),
        ("histogram", Some(matches)) => histogram(matches, output_method),
        ("mean", Some(matches)) => mean(matches, input_method),
        ("variance", Some(matches)) => variance(matches, input_method),
        _ => unreachable!(),
    }
}
