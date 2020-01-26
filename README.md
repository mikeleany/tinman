# Tinman Chess Engine

Tinman is a 64-bit computer chess engine developed by [Mike Leany](http://www.mikeleany.com/).
It is a re-write of my old chess engine, [Vapor](http://vapor.mikeleany.com/), into the Rust
programming language. As of v0.1.0, it plays about on par with Vapor, despite thinking more
slowly and lacking a transposition table. It appears that this is because Tinman has a better
evaluation function.

## Execution
To run the engine, you will probably want some kind of user interface. Once the arbiter tool is
completed, it can be used to run automated matches without a graphical interface. If you want
a graphical user interface, for instance to play human vs computer game, you will need one that
is compatible with the xboard protocol, such as
[XBoard/WinBoard](https://www.gnu.org/software/xboard/) or
[Arena](http://www.playwitharena.de/). Refer to the documentation for the specific user
interface for instructions on how to set up new engines. You will need to give the interface
the command to run which is simply the path to the executable.

## Additional Tools
 - `arbiter` will be a command-line tool meant to facilitate games between two chess engines.
 - [`count`](../count/index.html) is a tool which counts the variations from a position to a
   specified depth.
