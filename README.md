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
Minimum: -2.8826019900345274
Maximum: 3.3090509474620724
Mean: 0.00875870675676984
Variance: 1.0311933546275274
Standard deviation: 1.0154769099430707
Skewness: 0.054278758550605065
Kurtosis: 2.829776187145931
Population variance: 1.0291309679182723
Population standard deviation: 1.0144609247862986
Population skewness: 0.05411578638777538
Population kurtosis: 2.819497617893435

▶ samplers gaussian -N 500 | samplers histogram
Count: 500
Minimum: -3.5326608584379944
Maximum: 3.4345183423082233
Mean: -0.022511696583327738
Variance: 1.0236784268051349
Standard deviation: 1.011769947569671
Skewness: 0.008624017919115485
Kurtosis: 3.2441054641709943
Population variance: 1.0216310699515245
Population standard deviation: 1.0107576712306092
Population skewness: 0.008598124275080684
Population kurtosis: 3.229695186841172
   -inf │ 0
 -3.881 │▋ 1
 -3.370 │ 0
 -2.859 │▋ 1
 -2.348 │███████████▉ 18
 -1.837 │██████████████████████▍ 34
 -1.326 │██████████████████████████████████▍ 52
 -0.815 │██████████████████████████████████████████████▉ 71
 -0.305 │████████████████████████████████████████████████████████████████████████████████ 121
  0.206 │██████████████████████████████████████████████████████████▏ 88
  0.717 │█████████████████████████████████████████▋ 63
  1.228 │██████████████████████▍ 34
  1.739 │███████▎ 11
  2.250 │█▉ 3
  2.761 │▋ 1
  3.272 │█▎ 2
    inf │ 0

▶ samplers exponential --lambda 0.76 -N 500 | samplers histogram
Count: 500
Minimum: 0.0002634815470087787
Maximum: 9.104043860417937
Mean: 1.3830823293444519
Variance: 1.744680799765066
Standard deviation: 1.3208636567659306
Skewness: 1.7789645527962248
Kurtosis: 7.183990959944145
Population variance: 1.7411914381655358
Population standard deviation: 1.3195421320160776
Population skewness: 1.7736232054900543
Population kurtosis: 7.130291987295692
   -inf │ 0
 -0.455 │█████████████████████████████▏ 63
  0.213 │████████████████████████████████████████████████████████████████████████████████ 173
  0.880 │███████████████████████████████████████████████▋ 103
  1.548 │██████████████████████████▎ 57
  2.216 │████████████████████▊ 45
  2.883 │██████████▋ 23
  3.551 │█████ 11
  4.218 │██████ 13
  4.886 │█▊ 4
  5.554 │█▊ 4
  6.221 │▉ 2
  6.889 │▍ 1
  7.556 │ 0
  8.224 │ 0
  8.892 │▍ 1
    inf │ 
