# samplers

```shell
▶ samplers
samplers
Sample from common distributions and calculate summary statistics from the command line.

USAGE:
    samplers <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    binomial       Sample from a binomial distribution Bin(n, p)
    exponential    Sample from an exponential distribution Exp(λ)
    gaussian       Sample from a normal distribution 𝓝（μ, σ²）
    help           Prints this message or the help of the given subcommand(s)
    histogram      Displays a histogram of given values.
    mean           Calculate the mean of given values.
    poisson        Sample from a Poisson distribution Pois(λ)
    summarize      Calculate basic summary statistics.
    uniform        Sample from a uniform distribution Uniform(a, b)
    variance       Calculate the variance of given values.
```

## Usage

```shell
▶ samplers gaussian
0.16913471218719806

▶ samplers gaussian --mean -3.0 --variance 2.3
-3.997311502369134

▶ samplers gaussian -N 3
-0.46374056557817844
0.11965098764754963
0.0708432388236347

▶ samplers gaussian -N 500 | samplers variance
0.9902143982448738

▶ samplers gaussian -N 500 | samplers summarize
Count: 500
Minimum: -2.5872386009838197
Maximum: 3.1635978179614797
Mean: -0.01208947606813861
Variance: 0.9897750422237711
Sample variance: 0.991758559342456

▶ samplers gaussian -N 500 | samplers histogram
Count: 500
Minimum: -2.9518076550671117
Maximum: 3.2268658040411093
Mean: 0.05677536519757693
Variance: 0.9525602362033815
Sample variance: 0.9544691745524865
   -inf │ 0
 -3.261 │▊ 1
 -2.808 │█▋ 2
 -2.355 │███████▋ 9
 -1.901 │███████████████▍ 18
 -1.448 │███████████████████████████▌ 32
 -0.995 │████████████████████████████████████████████████████████████████████▊ 80
 -0.542 │██████████████████████████████████████████████████████████████████████████▊ 87
 -0.089 │████████████████████████████████████████████████████████████████████▊ 80
  0.364 │████████████████████████████████████████████████████████████████████████████████ 93
  0.817 │███████████████████████████████████████▌ 46
  1.270 │████████████████████████▉ 29
  1.723 │███████████▏ 13
  2.176 │███████▋ 9
  2.630 │ 0
  3.083 │▊ 1
    inf │ 0

▶ samplers exponential --lambda 0.76 -N 500 | samplers histogram
Count: 500
Minimum: 0.0035282136808685084
Maximum: 7.884792181221731
Mean: 1.3326301062331305
Variance: 1.6157151330938484
Sample variance: 1.6189530391721927
   -inf │ 0
 -0.391 │███████████████████████████████████████▏ 70
  0.187 │████████████████████████████████████████████████████████████████████████████████ 143
  0.765 │██████████████████████████████████████████████████▎ 90
  1.343 │█████████████████████████████████████████▍ 74
  1.921 │█████████████████████████████ 52
  2.499 │████████████▊ 23
  3.077 │███████████▏ 20
  3.655 │████▍ 8
  4.233 │████▍ 8
  4.811 │██▊ 5
  5.389 │█ 2
  5.967 │█▋ 3
  6.545 │ 0
  7.123 │▌ 1
  7.701 │▌ 1
    inf │ 0
```
