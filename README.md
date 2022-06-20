# Rusty Befunge

This is a small interpreter written in Rust for the 2-dimensional stack-based esoteric programming language [Befunge](https://esolangs.org/wiki/Befunge). The executable expects a single argument, which is the path to the source file.

## Compatibility

This interpreter approximately implements Befunge-93, the first version of the language, with the notable change that it uses a 256x256 character play area rather than the traditional 80x25. This makes the language implemented here a strict superset of Befunge-93. The [original language specification](https://catseye.tc/view/Befunge-93/doc/Befunge-93.markdown) also leaves several details regarding the handling of user input underspecified. In this implementation, input is buffered until a newline character is received, and numeric input is attempted to be parsed as a `u64` then wrapped to a `u8`. Invalid numeric input causes the program to panic.

## Sample programs

Several sample befunge programs are included in the `samples` folder, taken directly from the Esolang wiki.