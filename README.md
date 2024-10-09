## rscc

![Unit Tests](https://github.com/camertron/rscc/actions/workflows/test.yml/badge.svg?branch=main)

rscc is a compiler for the Reasonably Simple Computer (RSC) language. RSC was developed at Principia College in the early 1990s as a teaching tool for use in entry-level computer science courses. The compiler is capable of producing an executable from a text file containing RSC instructions.

## RSC

The original RSC interpreter was written in C, with a more [modern version](https://github.com/camertron/rsc.js) written in CoffeeScript in 2015. It is this modern version that powers https://reasonably-simple-computer.herokuapp.com, which also contains information about the instruction set, provides sample exercises, etc.

The rscc project is designed to supplement rsc.js, and was created more or less for fun. There is really no reason to compile RSC programs, as there is no reason to distribute them. I created the project as an excuse to play more with Rust (specifically the cranelift compiler backend), learn about cross-compilation, and generally as a learning exercise.

## Installation

Downloadable artifacts are provided for each version via GitHub releases. The latest release can be found here: https://github.com/camertron/rscc/releases/latest.

Cranelift, the underlying compiler backend rscc uses, only supports creating object files and does not support creating actual executables. For this reason, rscc requires a C toolchain to be available in your `PATH`.

### MacOS and Linux

For MacOS and Linux, releases include .tar.gz files for both x86 and ARM architectures. The .tar.gz files include the rscc executable and an rsc.c file containing I/O helper functions. The compiler builds rsc.c and includes it in compiled executables.

Installing a C toolchain for MacOS usually involves installing XCode or at least the XCode command-line tools. FreeCodeCamp has a [great writeup](https://www.freecodecamp.org/news/install-xcode-command-line-tools/) on how to do this.

On Linux, the easiest way to go is to install `gcc` via your distro's package manager. On Ubuntu for example, running `apt-get install gcc` should get you going.

### Windows

Windows is a bit of a special beast since Microsoft does not provide a low-impact or low-effort way to install a C toolchain, nor do most Windows machines have a package manager like [Chocolatey](https://chocolatey.org/) set up. For this reason, rscc provides a convenient Windows installer that includes a stripped-down version of [mingw64](https://www.mingw-w64.org/), a version of GCC compiled for Windows. The installer also makes the `rscc` command available on your `PATH`, meaning that after installation, everything should Just Work™.

In addition to the installer, rscc releases also include a .tar.gz file containing the stripped-down copy of mingw64, the rscc.exe executable, and the rsc.c helper file.

## Usage

rscc features a CLI that supports three subcommands: `build`, `run`, and `check`.

### Build

The `build` subcommand compiles an RSC program and produces an executable. You give it the path to a file and an optional path to an output directory. When compilation is finished, the output directory will contain a target/ directory containing temporary build artifacts, and an executable with the same base name as the input file. For example, if the input file is named "test.rsc", the executable will be named "test" (or "test.exe" on Windows platforms).

Let's compile the following example program that computes 5 / 2.

```ruby
LDC 5   # load constant 5 into the accumulator
STA 10  # store accumulator at memory address 10
LDC 2   # load constant 2 into the accumulator
STA 11  # store accumulator at memory address 11
LDA 10  # load contents of address 10 into the accumulator (value of 5)
DIV 11  # divide the accumulator by the contents of address 11 (value of 2)
STA 12  # store the result (i.e. the result of 5 / 2) into address 12
OUT 12  # output the value stored in address 12 (value of 2.5)
STP     # end program
```

To compile this program, run:

```bash
$> rscc build -f test.rsc
```

Then run the resulting executable. It should print `2.50` to standard output:

```bash
$> ./target/test/test
```

`build` exits with a status code of 0 if the program was compiled successfully, 1 if there were syntax errors, etc.

### Run

The `run` subcommand works similarly to `build`, but it doesn't produce an executable; instead, `run` executes the program directly, reading from standard input and writing to standard output. The following command should print `2.50` to standard output:

```bash
$> rscc run -f test.rsc
```

`run` exits with a status code of 0 if the program ran successfully, 1 if there were syntax errors, etc.

### Check

Finally, the `check` command validates an RSC program and prints out any problems it finds. For example, consider the following RSC program:

```ruby
LDC
STP
```

The `LDC` instruction requires a single operand, but none was provided. Let's run the `check` command on it:

```bash
$> rscc check -f test.rsc
```

The `check` command produces the following output:

```
Found 1 compilation problem(s)

-------------- PROBLEM 1 ---------------
1. LDC
      ^ Missing operand
2. STP
```

`check` exits with a status code of 0 if no problems were found, 1 otherwise.

## Running Tests

`cargo test` should do the trick.

## License

Licensed under the MIT license. See LICENSE for details.

## Authors

* Cameron C. Dutro: http://github.com/camertron
