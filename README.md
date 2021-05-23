# BFC
A BrainFuck compiler. Compiles to rust first, then invokes `rustc` to compile the intermeiate.
This application requires the rust toolchain and rustc installed to compile the source code.

The program can take multiple source code files and will combine them into one output

## Usage

`bfc <file> [...files] [-k]`
`file` - The main file to be compiled
`files` - The subsistent files you want compiled
`-k` - Keep the intermediate source code