use std::io::BufRead;

use clap::{value_t, App, AppSettings, Arg, SubCommand};
use rand::Rng;
use rand_distr::{Distribution, Normal, StandardNormal};

fn gaussian(mean: f64, variance: f64) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let normal = Normal::new(mean, variance.sqrt())?;
    Ok(normal.sample_iter(rand::thread_rng()))
}

fn standard_gaussian() -> impl Iterator<Item = f64> {
    rand::thread_rng().sample_iter(StandardNormal)
}

fn mean(
    mut values: impl Iterator<Item = Result<f64, failure::Error>>,
) -> Result<f64, failure::Error> {
    let (_count, mean) = values
        .try_fold::<_, _, Result<(u64, f64), failure::Error>>((0, 0.0), |(count, mean), r| {
            Ok((count + 1, mean + (r? - mean) / (count as f64 + 1.0)))
        })?;
    Ok(mean)
}

fn variance(
    mut values: impl Iterator<Item = Result<f64, failure::Error>>,
) -> Result<(f64, f64), failure::Error> {
    // An implementation of Welford's algorithm.
    let (count, _mean, sum_square_difference_from_mean) =
        values.try_fold::<_, _, Result<(u64, f64, f64), failure::Error>>(
            (0, 0.0, 0.0),
            |(mut count, mut mean, mut sum_square_difference_from_mean), r| {
                let v = r?;
                count += 1;
                let delta1 = v - mean;
                mean += delta1 / count as f64;
                let delta2 = v - mean;
                sum_square_difference_from_mean += delta1 * delta2;
                Ok((count, mean, sum_square_difference_from_mean))
            },
        )?;
    let population_variance = sum_square_difference_from_mean / count as f64;
    let sample_variance = sum_square_difference_from_mean / (count as f64 - 1.0);
    Ok((population_variance, sample_variance))
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
        ("gaussian", Some(matches)) => {
            let n = clap::value_t!(matches, "num_samples", usize)?;
            let mean = clap::value_t!(matches, "mean", f64)?;
            let variance = clap::value_t!(matches, "variance", f64)?;
            if mean == 0.0 && variance == 1.0 {
                standard_gaussian().take(n).for_each(|v| println!("{}", v));
            } else {
                gaussian(mean, variance)?
                    .take(n)
                    .for_each(|v| println!("{}", v));
            };
        }
        ("mean", Some(_matches)) => {
            println!("{}", mean(get_values_from_stdin(&mut std::io::stdin()))?);
        }
        ("variance", Some(matches)) => {
            let (population_variance, sample_variance) =
                variance(get_values_from_stdin(&mut std::io::stdin()))?;
            println!(
                "{}",
                match matches.value_of("type") {
                    Some("population") => population_variance,
                    Some("sample") => sample_variance,
                    _ => unreachable!(),
                }
            );
        }
        _ => unreachable!(),
    }
    Ok(())
}
