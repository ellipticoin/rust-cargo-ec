#[macro_use]
extern crate clap;
extern crate toml;
extern crate failure;

use clap::App;
use serde::Deserialize;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::io::Read;

fn main() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .setting(clap::AppSettings::AllowExternalSubcommands)
        .get_matches();
    match matches.subcommand_name() {
        Some("new") => new(matches
            .subcommand_matches("new")
            .unwrap()
            .value_of("name")
            .unwrap()),
        Some("build") => {
            if let Some(args) = matches
                .subcommand_matches("build")
                .unwrap()
                .values_of("cargo_args")
            {
                build(args.collect())
            } else {
                build(vec![])
            }
        }
        Some("deploy") => {
                let private_key = base64::decode(&matches.subcommand_matches("deploy")
                .unwrap()
                .value_of("private-key").unwrap()).expect("invalid private key");
                let path = &matches.subcommand_matches("deploy")
                    .unwrap()
                    .value_of("path")
                    .unwrap();
                let contract_name = &matches.subcommand_matches("deploy")
                    .unwrap()
                    .value_of("contract_name")
                    .unwrap();
                let constructor_arguments = {if let Some(args) = matches
                    .subcommand_matches("deploy")
                        .unwrap()
                        .values_of("constructor_arguments")
                        {
                            args.collect()
                        } else {
                            vec![]
                        }
                };

                deploy(contract_name, path, private_key, constructor_arguments)
        }
        _ => Ok(()),
    }
}

fn build(build_args: Vec<&str>) -> Result<(), Error> {
    let stdout = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .args(build_args)
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    Ok(())
}

fn deploy(
    contract_name: &str,
    path: &str,
    private_key: Vec<u8>,
    constructor_argument_strs: Vec<&str>
    ) -> Result<(), Error> {
    let mut f = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let constructor_arguments: Vec<ec_client::Value> = constructor_argument_strs
        .iter()
        .map(|argument| {
            if let Ok(number_argument) = argument.parse::<u64>() {
                ec_client::Value::U64(number_argument)
            } else if argument.starts_with("base64:") {
                let argument_bytes = base64::decode(argument.trim_start_matches("base64:")).unwrap();
                ec_client::Value::Bytes(argument_bytes)
            } else {
                ec_client::Value::String(argument.to_string())
            }
        }).collect();
    ec_client::create_contract(contract_name, &buffer, constructor_arguments, &private_key);
    println!("Created {}", contract_name);
    Ok(())
}

fn new(name: &str) -> Result<(), Error> {
    println!("Created {}", name);
    Ok(())
}
