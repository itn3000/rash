#[macro_use] extern crate clap;
extern crate failure;
extern crate md5;
extern crate blake2;
extern crate ripemd160;
extern crate digest;
extern crate sha1;
extern crate whirlpool;
extern crate sha2;
extern crate sha3;
extern crate groestl;

use failure::Error;
use clap::{Arg, SubCommand, AppSettings, ArgMatches};
use digest::{Digest, VariableOutput, FixedOutput, Input, ExtendableOutput, XofReader};

use std::io::{Read, BufReader};
use std::fs::File;

fn run() -> Result<(), Error> {
    let matches = args();
    let mut file = BufReader::new(File::open(matches.value_of("FILE").ok_or_else(|| failure::err_msg("missing file"))?)
        .map_err(Error::from)?);
    let hash = get_alg(&matches, &mut file)?;

    println!("{}", hash);

    Ok(())
}

fn calc_hash_fixed<D, R>(mut digest: D, input: &mut R) -> Result<String, Error> where D: FixedOutput + Input, R: Read {
    fill(&mut digest, input)?;
    Ok(digest.fixed_result().iter().map(|x| format!("{:02x}", x)).collect::<String>())
}

fn calc_hash_var<D, R>(mut digest: D, input: &mut R) -> Result<String, Error> where D: VariableOutput + Input, R: Read {
    fill(&mut digest, input)?;
    let mut buff = Vec::with_capacity(digest.output_size());
    digest.variable_result(&mut buff).map_err(|_| failure::err_msg("invalid length"))?;
    Ok(buff.iter().map(|x| format!("{:02x}", x)).collect::<String>())
}

fn calc_hash_extendable<D, R>(mut digest: D, input: &mut R, len: usize) -> Result<String, Error> where D: ExtendableOutput + Input, R: Read {
    fill(&mut digest, input)?;
    let mut buff = vec![0; len];
    digest.xof_result().read(&mut buff);
    Ok(buff.iter().map(|x| format!("{:02x}", x)).collect::<String>())
}

fn fill<D, R>(digest: &mut D, input: &mut R) -> Result<(), Error> where D: Input, R: Read {
    let mut buf = [0 as u8; 1024];
    loop {
        if input.read(&mut buf).map_err(Error::from)? == 0 {
            break;
        } else {
            digest.process(&buf);
        }
    };
    Ok(())
}

fn args<'a>() -> ArgMatches<'a> {
    app_from_crate!()
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("md5")
            .about("MD5 algorithm")
            .long_about("MD5 algorithm. This algorithm has known vulnerabilities, use a more modern algorithm."))
        .subcommand(SubCommand::with_name("whirlpool")
            .about("whirlpool algorithm"))
        .subcommand(SubCommand::with_name("sha1")
            .about("SHA1 algorithm")
            .long_about("SHA1 algorithm. This algorithm has known vulnerabilities, use a more modern algorithm."))
        .subcommand(SubCommand::with_name("ripemd160")
            .about("Ripemd160 algorithm"))
        .subcommand(SubCommand::with_name("blake2b")
            .about("BLAKE2b algorithm")
            .long_about("BLAKE2b algorithm. Recommended over BLAKE2s on 64bit platforms.")
            .arg(Arg::with_name("len")
                .short("l")
                .long("length")
                .help("Length of output hash in bytes")
                .long_help("Length of output hash in bytes. Must be between 1 & 64 inclusive.")
                .default_value("64")))
        .subcommand(SubCommand::with_name("blake2s")
            .about("BLAKE2s algorithm")
            .long_about("BLAKE2s algorithm. Recommended over BLAKE2b on 32bit platforms.")
            .arg(Arg::with_name("len")
                .short("l")
                .long("length")
                .help("Length of output hash in bytes")
                .long_help("Length of output hash in bytes. Must be between 1 & 32 inclusive.")
                .default_value("32")))
        .subcommand(SubCommand::with_name("sha2")
            .about("SHA2 algorithms")
            .long_about("SHA2 Algorithms. Supports SHA256 (truncated to 224, 256) and SHA512 (truncated to 224, 256, 384, 512).")
            .arg(Arg::with_name("len")
                .short("l")
                .long("length")
                .help("Length of output hash")
                .long_help("Length of the output hash. Supported lengths with algorithms:\
                \n\talg:\tlen\
                \n\t256:\t224, 256\
                \n\t512:\t224, 256, 384, 512\n")
                .possible_values(&["224", "256", "384", "512"])
                .default_value_ifs(&[("alg", Some("512"), "512"),("alg", Some("256"), "256")])
                .takes_value(true))
            .arg(Arg::with_name("alg")
                .short("a")
                .long("algorithm")
                .help("SHA2 algorithm")
                .takes_value(true)
                .default_value("512")
                .possible_values(&["256", "512"])))
        .subcommand(SubCommand::with_name("sha3")
            .about("SHA3 algorithms")
            .arg(Arg::with_name("len")
                .short("l")
                .long("length")
                .help("Length of output hash")
                .long_help("Length of the output hash. Supported lengths with algorithms:\
                \n\talg:\tlen\
                \n\tsha3:\t244, 256, 384, 512\
                \n\tkeccak:\t244, 256, 384, 512\n")
                .default_value_ifs(&[("alg", Some("sha3"), "512"), ("alg", Some("keccak"), "512")])
                .takes_value(true))
            .arg(Arg::with_name("alg")
                .short("a")
                .long("algorithm")
                .help("SHA3 Algorithm")
                .long_help("The SHA3 algorithm to use. If not given, sha3 is assumed. See len's help for length algorithm combinations.")
                .takes_value(true)
                .possible_values(&["sha3", "keccak"])
                .default_value("sha3")))
        .subcommand(SubCommand::with_name("shake")
            .about("Shake algorithm")
            .arg(Arg::with_name("len")
                .short("l")
                .long("length")
                .help("Length of output hash")
                .long_help("Length of output hash. Can be any positive integer.")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("var")
                .short("v")
                .long("variant")
                .help("Shake variant")
                .possible_values(&["128", "256"])
                .default_value("256")))
        .subcommand(SubCommand::with_name("groestl")
            .about("Groestl Algorithm")
            .arg(Arg::with_name("len")
                .short("l")
                .long("length")
                .help("Length of output hash")
                .long_help("Length of output. Must be between 1 & 64 inclusive.")
                .default_value("64")
                .takes_value(true)))
        .arg(Arg::with_name("FILE")
            .help("File to calculate the hash of")
            .global(true))
        .get_matches()
}

fn get_alg<'a, R>(matches: &ArgMatches<'a>, input: &mut R) -> Result<String, Error> where R: Read {
    match matches.subcommand() {
        ("md5", _) => calc_hash_fixed(md5::Md5::new(), input),
        ("whirlpool", _) => calc_hash_fixed(whirlpool::Whirlpool::new(), input),
        ("sha1", _) => calc_hash_fixed(sha1::Sha1::new(), input),
        ("ripemd160", _) => calc_hash_fixed(ripemd160::Ripemd160::new(), input),
        ("blake2b", Some(matches)) => {
            let len = matches.value_of("len").unwrap().parse().map_err(Error::from)?;
            calc_hash_fixed(<blake2::Blake2b as VariableOutput>::new(len)
                          .map_err(|_| failure::err_msg("invalid length"))?, input)
        },
        ("blake2s", Some(matches)) => {
            let len = matches.value_of("len").unwrap().parse().map_err(Error::from)?;
            calc_hash_fixed(<blake2::Blake2s  as VariableOutput>::new(len)
                          .map_err(|_| failure::err_msg("invalid length"))?, input)
        },
        ("sha2", Some(matches)) => {
            match matches.value_of("alg").unwrap().parse().map_err(Error::from)? {
                256 => match matches.value_of("len").unwrap().parse().map_err(Error::from)? {
                    224 => calc_hash_fixed(sha2::Sha224::new(), input),
                    256 => calc_hash_fixed(sha2::Sha256::new(), input),
                    _ => Err(failure::err_msg("invalid length for SHA2-256"))
                },
                512 => match matches.value_of("len").unwrap().parse().map_err(Error::from)? {
                    224 => calc_hash_fixed(sha2::Sha512Trunc224::new(), input),
                    256 => calc_hash_fixed(sha2::Sha512Trunc256::new(), input),
                    384 => calc_hash_fixed(sha2::Sha384::new(), input),
                    512 => calc_hash_fixed(sha2::Sha512::new(), input),
                    _ => Err(failure::err_msg("invalid length for SHA2-256"))
                },
                _ => Err(failure::err_msg("invalid SHA2 algorithm"))
            }
        },
        ("shake", Some(matches)) => {
            match matches.value_of("var").unwrap_or_else(|| "512").parse().map_err(Error::from)? {
                128 => calc_hash_extendable(sha3::Shake128::default(),
                                            input,
                                            matches.value_of("len")
                                                .ok_or_else(|| failure::err_msg("missing length"))?
                                                .parse()
                                                .map_err(Error::from)?),
                256 => calc_hash_extendable(sha3::Shake256::default(),
                                            input,
                                            matches.value_of("len")
                                                .ok_or_else(|| failure::err_msg("missing length"))?
                                                .parse()
                                                .map_err(Error::from)?),
                _ => Err(failure::err_msg("invalid variant"))
            }
        },
        ("sha3", Some(matches)) => {
            match matches.value_of("alg").unwrap_or_else(|| "sha3") {
                "sha3" => match matches.value_of("len").unwrap_or_else(|| "512").parse().map_err(Error::from)? {
                    224 => calc_hash_fixed(sha3::Sha3_224::new(), input),
                    256 => calc_hash_fixed(sha3::Sha3_256::new(), input),
                    384 => calc_hash_fixed(sha3::Sha3_384::new(), input),
                    512 => calc_hash_fixed(sha3::Sha3_512::new(), input),
                    _ => Err(failure::err_msg("invalid length for SHA3"))
                },
                "keccak" => match matches.value_of("len").unwrap_or_else(|| "512").parse().map_err(Error::from)? {
                    224 => calc_hash_fixed(sha3::Keccak224::new(), input),
                    256 => calc_hash_fixed(sha3::Keccak256::new(), input),
                    384 => calc_hash_fixed(sha3::Keccak384::new(), input),
                    512 => calc_hash_fixed(sha3::Keccak512::new(), input),
                    _ => Err(failure::err_msg("invalid length for Keccak"))
                }
                _ => Err(failure::err_msg("invalid SHA3 algorithm"))
            }
        },
        ("groestl", Some(matches)) => {
            let len = matches.value_of("len").unwrap().parse().map_err(Error::from)?;
            if len > 0 && len < 33 {
                calc_hash_var(groestl::GroestlSmall::new(len).map_err(|_| failure::err_msg("invalid length"))?, input)
            } else if len > 32 && len < 65 {
                calc_hash_var(groestl::GroestlBig::new(len).map_err(|_| failure::err_msg("invalid length"))?, input)
            } else {
                Err(failure::err_msg("invalid length for Groestl"))
            }
        }
        _ => Err(failure::err_msg("unknown algorithm"))
    }
}

fn main() {
    match run() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}", e);
            ::std::process::exit(1)
        }
    }
}