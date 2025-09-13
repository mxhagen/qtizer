
# 🎨 qtizer

command line quantization/palette-generation tool using k-means clustering on pixel data

- [features](#features)
- [usage](#usage)
- [installation](#installation)


## features

- hex and rgb formats
- output with color previews
- various supported file types


## usage

```
Usage: qtizer [OPTIONS] <input> [output]

Arguments:
  <input>   Input file path
  [output]  Output file path

Options:
  -k <count>             Number of colors to quantize to [default: 8]
  -n <count>             Number of k-means iterations to perform [default: 5]
  -a, --with-alpha       Include alpha channel
  -s, --seed <number>    Optional RNG seed for reproducible results
  -o, --output <output>  Output file path
                         - If not provided, outputs to stdout
  -f, --format <fmt>     Palette output format [default: hex] [possible values: hex, rgb]
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

Example (output is colored accordingly in terminals):
```sh
$ qtizer wallpaper.png -k 3 -af rgb

rgba(254, 254, 254, 0)
rgba(191, 150, 132, 254)
rgba(48, 45, 51, 254)
```


## installation

```sh
git clone https://github.com/mxhagen/qtizer
```
```sh
cd qtizer; cargo build --release
```
```sh
sudo cp target/release/qtizer /usr/local/bin
```

