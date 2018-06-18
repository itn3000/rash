# Rash
## A file hashing tool written in Rust

Command line application for calculating the hash value of a file. Rash uses the [rust-crypto](https://crates.io/crates/rust-crypto) library, therefore currently supports
* md5
* SHA1
* Whirlpool
* Ripemd160
* Blake2b
* Blake2S
* SHA2-256 (truncated to 224 and 256)
* SHA2-512 (truncated to 224, 256, 384 and 512)
* SHA3 (truncated to 224, 256, 384 and 512)
* Shake-128 
* Shake-256
* Keccak (truncated to 224, 256, 384 and 512)

The [rust-crypto](https://crates.io/crates/rust-crypto) library currently has not been throughly audited for correctness as of yet, so do not use Rash if the correctness of the hashes is important.

## Usage
The Shake and Keccak are accessed with the sha3 subcommand.

    USAGE:
        rash [FILE] <SUBCOMMAND>
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    ARGS:
        <FILE>    File to calculate the hash of
    SUBCOMMANDS:
        blake2b      BLAKE2b algorithm
        blake2s      BLAKE2s algorithm
        help         Prints this message or the help of the given subcommand(s)
        md5          md5 algorithm
        ripemd160    Ripemd160 algorithm
        sha1         SHA1 algorithm
        sha2         SHA2 algorithms
        sha3         SHA3 algorithms
        whirlpool    whirlpool algorithm

## Examples
    rash md5 Readme.md
    rash sha3 Readme.md
    rash blake2b -l 64 Readme.md

## Crates used
* [rust-crypto](https://crates.io/crates/rust-crypto) 
* [clap](https://crates.io/crates/clap)
* [failure](https://crates.io/crates/failure)