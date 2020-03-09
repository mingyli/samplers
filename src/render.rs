use std::io::Write;

use crate::histogram::Bucket;

pub fn render_buckets(
    buckets: &[Bucket],
    display_size: usize,
    mut output: impl Write,
) -> Result<(), failure::Error> {
    use itertools::{Itertools, Position};

    let max_count = buckets
        .iter()
        .map(|bucket| bucket.count())
        .max()
        .ok_or_else(|| format_err!("there are buckets"))?;

    for elem in buckets.iter().with_position() {
        let bucket = elem.into_inner();
        let proportion: f64 = bucket.count() as f64 / max_count as f64;
        let num_chars: f64 = display_size as f64 * proportion;
        writeln!(
            output,
            "{:>7.3} │{} {}",
            bucket.lower(),
            format!("{}{}", "█".repeat(num_chars.floor() as usize), {
                render_fraction_bar(num_chars.fract())
            }),
            bucket.count(),
        )?;
        match elem {
            Position::First(_) | Position::Middle(_) => {}
            Position::Only(bucket) | Position::Last(bucket) => {
                writeln!(output, "{:>7.3} │ 0", bucket.upper())?;
            }
        }
    }

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
