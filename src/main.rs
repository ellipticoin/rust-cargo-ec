#[macro_use] extern crate clap;
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
    let ec_matches = matches.subcommand_matches("ec").unwrap();
    match ec_matches.subcommand_name() {
        Some("new") => new(ec_matches
            .subcommand_matches("new")
            .unwrap()
            .value_of("name")
            .unwrap()),
        Some("build") => {
            if let Some(args) = ec_matches
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
                let private_key = base64::decode(&ec_matches.subcommand_matches("deploy")
                .unwrap()
                .value_of("private-key").unwrap()).expect("invalid private key");
                let path = &ec_matches.subcommand_matches("deploy")
                    .unwrap()
                    .value_of("path")
                    .unwrap();
                let contract_name = &ec_matches.subcommand_matches("deploy")
                    .unwrap()
                    .value_of("contract_name")
                    .unwrap();
                let constructor_arguments = {if let Some(args) = ec_matches
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
        .arg("--release")
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

    snip();
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Config {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
}

fn snip() {
        let config: Config = toml::from_str(&std::fs::read_to_string("Cargo.toml").unwrap()).unwrap();
        let mut path = std::path::PathBuf::new();
        path.push("target");
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(config.package.name.clone());
        path.set_extension("wasm");
        let output = wasm_snip::snip(wasm_snip::Options {
            input: path.clone(),
            functions: vec![],
            patterns: vec![],
            snip_rust_fmt_code: true,
            snip_rust_panicking_code: true,
            skip_producers_section: false
        }).unwrap();
        path.pop();
        path.push([config.package.name.clone(), "-min".to_string()].concat());
        path.set_extension("wasm");
    output.emit_wasm_file(path).unwrap();
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
    let cargo_path = [name, "Cargo.toml"].iter().collect::<std::path::PathBuf>().into_os_string().into_string().unwrap();
    let src_path = [name, "src"].iter().collect::<std::path::PathBuf>().into_os_string().into_string().unwrap();
    let lib_path = [name, "src", "lib.rs"].iter().collect::<std::path::PathBuf>().into_os_string().into_string().unwrap();
    let error_path = [name, "src", "error.rs"].iter().collect::<std::path::PathBuf>().into_os_string().into_string().unwrap();
    let main_path = [name, "src", &[name, ".rs"].join("")].iter().collect::<std::path::PathBuf>().into_os_string().into_string().unwrap();
    std::fs::create_dir(name).unwrap();
    std::fs::create_dir(src_path).unwrap();
    let cargo_str = include_str!("template/Cargo.toml.txt");
    let  lib_str = include_str!("template/lib.rs");
    let  main_str = include_str!("template/main.rs");
    let error_str = include_str!("template/error.rs");

    std::fs::write(cargo_path, render_template(cargo_str, name)).unwrap();
    std::fs::write(lib_path, render_template(lib_str, name)).unwrap();
    std::fs::write(main_path, render_template(main_str, name)).unwrap();
    std::fs::write(error_path, error_str).unwrap();

    println!("Created {}", name);
    Ok(())
}

fn render_template(s: &str, package_name: &str) -> String {
    s.replace("$PACKAGE_NAME", package_name)
}
