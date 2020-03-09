#[macro_use]
extern crate failure;

use std::io::BufRead;

use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};

mod distributions;
mod histogram;
mod summary;

use histogram::{Bucket, Histogram};
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

fn render_fraction_bar(frac: f64) -> &'static str {
    if frac > 7.0 / 8.0 {
        "▉"
    } else if frac > 6.0 / 8.0 {
        "▊"
    } else if frac > 5.0 / 8.0 {
        "▋"
    } else if frac > 4.0 / 8.0 {
        "▌"
    } else if frac > 3.0 / 8.0 {
        "▍"
    } else if frac > 2.0 / 8.0 {
        "▎"
    } else if frac > 1.0 / 8.0 {
        "▏"
    } else {
        ""
    }
}

fn histogram(matches: &ArgMatches) -> Result<(), failure::Error> {
    let values: Vec<f64> = get_values_from_stdin()?;
    let mut summary = DistributionSummary::default();
    summary.observe_many(values.iter())?;
    println!("{}", summary);

    const PADDING: f64 = 0.05;
    let num_buckets: usize = clap::value_t!(matches, "num_buckets", usize)?;
    let max = summary
        .max()
        .ok_or_else(|| SamplersError::CouldNotCalculateSummaryStatistic {
            name: "max".to_string(),
        })?;
    let min = summary
        .min()
        .ok_or_else(|| SamplersError::CouldNotCalculateSummaryStatistic {
            name: "min".to_string(),
        })?;
    let width: f64 = max - min;
    let bucket_width: f64 = (1.0 + PADDING + PADDING) * width / num_buckets as f64;
    let boundaries: Vec<f64> = (0..num_buckets)
        .map(|i| min - PADDING * width + (i as f64) * bucket_width)
        .collect();
    let mut histogram = Histogram::new(boundaries);
    histogram.observe_many(values.iter())?;
    let buckets = histogram.collect();

    render_buckets(matches, &buckets)
}

fn render_buckets(matches: &ArgMatches, buckets: &[Bucket]) -> Result<(), failure::Error> {
    use itertools::{Itertools, Position};

    let max_count = buckets
        .iter()
        .map(|bucket| bucket.count())
        .max()
        .ok_or_else(|| format_err!("there are buckets"))?;
    let display_size: usize = clap::value_t!(matches, "display_size", usize)?;

    for elem in buckets.iter().with_position() {
        let bucket = elem.into_inner();
        let proportion: f64 = bucket.count() as f64 / max_count as f64;
        let num_chars: f64 = display_size as f64 * proportion;
        println!(
            "{:>7.3} │{} {}",
            bucket.lower(),
            format!("{}{}", "█".repeat(num_chars.floor() as usize), {
                render_fraction_bar(num_chars.fract())
            }),
            bucket.count(),
        );
        match elem {
            Position::First(_) | Position::Middle(_) => {}
            Position::Only(bucket) | Position::Last(bucket) => {
                println!("{:>7.3} │ 0", bucket.upper());
            }
        }
    }

    Ok(())
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
            SubCommand::with_name("histogram")
                .about("Displays a histogram of given values.")
                .after_help("This reads from stdin. You can terminate stdin with CTRL+D.")
                .arg(
                    Arg::with_name("num_buckets")
                        .short("b")
                        .long("num_buckets")
                        .help("The number of buckets in the histogram.")
                        .default_value("15")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("display_size")
                        .short("d")
                        .long("display_size")
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

    match app_matches.subcommand() {
        ("gaussian", Some(matches)) => gaussian(matches),
        ("poisson", Some(matches)) => poisson(matches),
        ("exponential", Some(matches)) => exponential(matches),
        ("uniform", Some(matches)) => uniform(matches),
        ("binomial", Some(matches)) => binomial(matches),
        ("summarize", Some(matches)) => summarize(matches, input_method),
        ("histogram", Some(matches)) => histogram(matches),
        ("mean", Some(matches)) => mean(matches, input_method),
        ("variance", Some(matches)) => variance(matches, input_method),
        _ => unreachable!(),
    }
}
