#  <img src="images/logo2.png" alt="Logo" width="50" height="25"> BasicRS

BasicRS is a Rust implementation of a BASIC Interpreter.

BasicRS is part of the TrekBasic family of BASIC programming tools.
* TrekBasic - Python version
* TrekBasicJ - Java Version
* BasicRS - Rust version
* BasicTestSuite - A test suite of BASIC Programs
* TrekBot - A tool to exercise the superstartrek program

All versions are intended to by byte-by-byte compatible, but are not
there yet - but they are close. TrekBot and BasicTestSuite are part of
plan to ensure full compabtiblity.

TrekBasic and TrekBasicJ are also compilers, and the compatibility
targets are the same for the compiled versions. A compiler for BasicRS is planned.

My goal was to be able to play the old Star Trek game, which was written in BASIC.

    https://en.wikipedia.org/wiki/Star_Trek_(1971_video_game). 

I have achieved that goal.

## To Run

cargo run -- superstartrek.bas

OR

target/debug/basic_rs superstartrek.bas

## Shell
If you want to use the shell for BASIC which is the command line "IDE" - sort of.

It provides breakpoints, single stepping, code coverage, and more.  

./target/debug/basic_shell superstartrek.bas

or just

./target/debug/basic_shell
