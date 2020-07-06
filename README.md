# Tinman
*A Rusty chess engine*

<!--
Copyright 2020 Michael Leany

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
-->

Tinman is a chess engine by [Mike Leany](http://www.mikeleany.com/), written in Rust and loosely
based on his previous chess engine [Vapor](https://github.com/mikeleany/vapor), which was
written in C. It currently supports the Chess Engine Communication Protocol (XBoard/WinBoard).
UCI support will be coming in a future release.

## Installation
The latest release can be downloaded from the
[Releases](https://github.com/mikeleany/tinman/releases) page on GitHub. If you are using a
pre-compiled executable, unzip it to wherever you want to run it from and skip to the next section.

If you are compiling from source, unzip the source from the releases page, or clone the
repository. In a terminal, go to the directory where you put the source code and run the
following command.

```shell
cargo build --release
```

The executable will be located in "target/release/" within the directory where the source code
is. In UNIX-like operating systems it should be named "tinman"; in Windows it will be
"tinman.exe".

### A User Interface
To run the engine, you will probably want some kind of graphical user interface.
Two possibilities are [XBoard/WinBoard](https://www.gnu.org/software/xboard/) or
[Arena](http://www.playwitharena.de/). If you use another interface, just make sure it supports
the XBoard protocol. Refer to the documentation for the specific user interface you choose for
instructions on how to set up new engines. You will need to give the interface the command to
run which, in the simplest case, is just the path of the executable.

## Execution
```text
tinman [OPTIONS] [SUBCOMMAND]
```

### Options and Flags
| Short | Long          | Arg | Description
|-------|---------------|-----|---------------------------------------------------------------------
| `-h`  | `--help`      | No  | Prints help information
| `-V`  | `--version`   | No  | Prints version information
|       | `--log-file`  | Yes | Sets the log file if logging is turned on (default: "tinman.log")
| `-l`  | `--log-level` | Yes | Sets the log level or turns off logging (default: `info`)

The log level can be any of the following, from the least verbose to the most:
`off`, `error`, `warn`, `info`, `debug`, or `trace`.

### The `counts` Subcommand
The `counts` subcommand is used to count the number of variations to a specific depth from one
or more positions. It is called perft in most chess engines. The command looks like the
following.

```shell
tinman counts -d <DEPTH> [FEN_STRING]...
```

`DEPTH` is required, but `FEN_STRING` is optional. If `FEN_STRING` is not given, counts the
variations from the standard starting position. `FEN_STRING` must be passed as a single
argument, which means it should be enclosed in quotes. If multiple FEN strings are passed, each
is counted in turn.

### Examples
Run with logging turned off:
```shell
tinman -l off
```

Run with maximum logging sent to a file named "verbose.log":
```shell
tinman -l trace --log-file verbose.log
```

Count the variations to depth of 3 for the standard starting position:
```shell
tinman counts -d 3
```

Count the variations to depth 6 for two different positions:
```shell
tinman counts -d 6 "4k3/8/8/8/8/8/8/4K2R w K - 0 1" "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1"
```
