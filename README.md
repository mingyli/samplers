# samplers

[![Crates.io](https://img.shields.io/crates/v/samplers.svg)](https://crates.io/crates/samplers)

`samplers` allows you to sample from common distributions and calculate
summary statistics from the command line.

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

### Sample from distributions

```shell
▶ samplers gaussian
0.16913471218719806

▶ samplers poisson --lambda 0.46
3

▶ samplers gaussian -N 3
-0.46374056557817844
0.11965098764754963
0.0708432388236347
```

### Calculate summary statistics

```shell
▶ samplers gaussian -N 500 | samplers variance
0.9902143982448738

▶ samplers poisson --lambda 0.76 -N 500 | samplers summarize
Count: 500
Minimum: 0
Maximum: 4
Mean: 0.7919999999999995
Variance: 0.794324649298597
Standard deviation: 0.8912489266745834
Skewness: 1.1363753055124572
Kurtosis: 4.144239039703653
Population variance: 0.7927359999999999
Population standard deviation: 0.8903572316772633
Population skewness: 1.1329633346739394
Population kurtosis: 4.120852594453947
```

### Generate histograms

```shell
▶ samplers gaussian -N 5000 | samplers histogram
   -inf │ 0
 -4.308 │ 1
 -3.749 │▎ 4
 -3.191 │█▉ 27
 -2.632 │██████▎ 85
 -2.074 │████████████████▍ 223
 -1.516 │███████████████████████████████████████ 530
 -0.957 │██████████████████████████████████████████████████████████████████▌ 903
 -0.399 │████████████████████████████████████████████████████████████████████████████████ 1086
  0.160 │██████████████████████████████████████████████████████████████████████████▉ 1017
  0.718 │██████████████████████████████████████████████▊ 636
  1.276 │███████████████████████▊ 323
  1.835 │████████▊ 119
  2.393 │██▌ 35
  2.952 │▋ 9
  3.510 │▏ 2
    inf │ 0
```

### Combine `samplers` commands

```shell
▶ samplers exponential -N 5000 | samplers histogram | samplers summarize
   -inf │ 0
 -0.500 │███████████████████████████████████████████ 1102
  0.234 │████████████████████████████████████████████████████████████████████████████████ 2046
  0.969 │██████████████████████████████████████▎ 981
  1.703 │████████████████▊ 430
  2.438 │█████████▏ 235
  3.172 │████▏ 106
  3.907 │█▉ 51
  4.641 │▉ 24
  5.375 │▌ 14
  6.110 │▏ 6
  6.844 │ 2
  7.579 │ 2
  8.313 │ 0
  9.048 │ 0
  9.782 │ 1
    inf │ 0
Count: 5000
Minimum: 0.0006669581918233141
Maximum: 10.015754597795226
Mean: 0.9826240271903571
Variance: 1.001426624099917
Standard deviation: 1.0007130578242283
Skewness: 2.0408990346110265
Kurtosis: 9.207233082818302
Population variance: 1.001226338775097
Population standard deviation: 1.00061298151438
Population skewness: 2.0402867138710232
Population kurtosis: 9.19982782746453

▶ ( samplers exponential -l 0.5 -N 500 & samplers gaussian -m -2.5 -N 500; ) | samplers histogram | samplers summarize
   -inf │ 0
 -6.129 │█▊ 5
 -4.910 │██████████████████▌ 51
 -3.691 │█████████████████████████████████████████████████████████████████████████████▊ 214
 -2.472 │██████████████████████████████████████████████████████████████▏ 171
 -1.253 │███████████████████▎ 53
 -0.033 │████████████████████████████████████████████████████████████████████████████████ 220
  1.186 │████████████████████████████████████████████████▎ 133
  2.405 │████████████████████████ 66
  3.624 │██████████████▉ 41
  4.843 │███████▋ 21
  6.063 │████ 11
  7.282 │██▉ 8
  8.501 │█ 3
  9.720 │▋ 2
 10.939 │▎ 1
    inf │ 0
Count: 1000
Minimum: -5.298192494925765
Maximum: 11.327405812322485
Mean: -0.2429294321045422
Variance: 7.635620235863008
Standard deviation: 2.7632626071119275
Skewness: 0.7453924717583317
Kurtosis: 3.4981671751521244
Population variance: 7.6279846156271445
Population standard deviation: 2.7618806302277337
Population skewness: 0.7442739168540279
Population kurtosis: 3.48968581796523
```
