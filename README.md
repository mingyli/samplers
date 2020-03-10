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
  0.000 │████████████████████████████████████████████████████████████████████████████████ 2175
  0.561 │█████████████████████████████████████████████▏ 1230
  1.121 │█████████████████████████▋ 699
  1.682 │██████████████ 382
  2.242 │████████ 218
  2.803 │████▌ 123
  3.363 │██▋ 74
  3.924 │█▌ 44
  4.484 │▊ 21
  5.044 │▎ 10
  5.605 │▎ 9
  6.165 │▎ 7
  6.726 │▏ 5
  7.286 │ 2
  7.847 │ 0
  8.407 │ 1
    inf │ 0
Count: 5000
Minimum: 0.00032381898365838605
Maximum: 8.40719489137377
Mean: 0.9864709169141752
Variance: 0.9942448035167946
Standard deviation: 0.9971182495154698
Skewness: 2.05849955673295
Kurtosis: 9.09039154352979
Population variance: 0.9940459545560912
Population standard deviation: 0.9970185327044283
Population skewness: 2.0578819553962355
Population kurtosis: 9.083103097004358

▶ ( samplers exponential -l 0.5 -N 500 & samplers gaussian -m -2.5 -N 500; ) | samplers histogram | samplers summarize
   -inf │ 0
 -5.694 │███▊ 11
 -4.444 │██████████████████████████████████▎ 99
 -3.193 │████████████████████████████████████████████████████████████████████████████████ 231
 -1.942 │██████████████████████████████████████████████ 133
 -0.691 │█████████████████████████████████████████████████████████▏ 165
  0.559 │████████████████████████████████████████████████████████████▎ 174
  1.810 │███████████████████████████████▊ 92
  3.061 │████████████████▎ 47
  4.311 │████████▎ 24
  5.562 │██▊ 8
  6.813 │██▊ 8
  8.064 │█▋ 5
  9.314 │ 0
 10.565 │▋ 2
 11.816 │ 0
 13.066 │▎ 1
    inf │ 0
Count: 1000
Minimum: -5.6942899675153615
Maximum: 13.066364835816431
Mean: -0.2723374142039541
Variance: 6.8256155608507685
Standard deviation: 2.6125879049040184
Skewness: 0.8726798865143978
Kurtosis: 4.3451283901570665
Population variance: 6.818789945289917
Population standard deviation: 2.6112812842146895
Population skewness: 0.8713703208775927
Population kurtosis: 4.332418151394773
```
