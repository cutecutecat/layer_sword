use std::path::Path;
use std::collections::HashMap;

use clap::{Arg, App, SubCommand, ArgGroup, ArgMatches, AppSettings};
use json::JsonValue;

use crate::split::split_layer;
use crate::merge::merge_layer;
use crate::util::{load_config, init_path};
use crate::errors::TerminalError;


fn parse_path<'a>(sub: &'a ArgMatches, mode: &str)
                  -> Result<(&'a Path, &'a Path, &'a Path), TerminalError> {
    let target = sub.value_of("target")
        .ok_or_else(|| TerminalError::WithoutArgError {
            arg: format!("target"),
            msg: sub.usage().to_string(),
        })?;
    let work = sub.value_of("work")
        .ok_or_else(|| TerminalError::WithoutArgError {
            arg: format!("work"),
            msg: sub.usage().to_string(),
        })?;
    let out = sub.value_of("output")
        .ok_or_else(|| TerminalError::WithoutArgError {
            arg: format!("target"),
            msg: sub.usage().to_string(),
        })?;

    let target_path = Path::new(target);
    let work_path = Path::new(work);
    let out_path = Path::new(out);

    if !target_path.exists() {
        return Err(TerminalError::NotExistError { path: target.to_string() });
    }
    if !target_path.is_file() && mode == "split" {
        return Err(TerminalError::NotFileError { path: target.to_string() });
    } else if !target_path.is_dir() && mode == "merge" {
        return Err(TerminalError::NotDirectoryError { path: target.to_string() });
    }
    if !out_path.exists() {
        return Err(TerminalError::NotExistError { path: target.to_string() });
    }
    if !out_path.is_dir() {
        return Err(TerminalError::NotDirectoryError { path: target.to_string() });
    }
    Ok((target_path, work_path, out_path))
}

fn parse_cfg_from_file(sub: &ArgMatches)
                       -> Result<(Vec<String>, HashMap<String, i16>), TerminalError> {
    let config = sub.value_of("config")
        .ok_or_else(|| TerminalError::WithoutArgError {
            arg: format!("config"),
            msg: sub.usage().to_string(),
        })?;
    let config_path = Path::new(config);
    if !config_path.exists() {
        return Err(TerminalError::NotExistError { path: config.to_string() });
    }
    if !config_path.is_file() {
        return Err(TerminalError::NotFileError { path: config.to_string() });
    }
    let client_config = load_config(config_path)
        .map_err(|_| TerminalError::InputConfigError)?;

    let names = match &client_config["names"] {
        JsonValue::Array(n) => { Ok(n) }
        _ => { Err(TerminalError::InputConfigError) }
    }?.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    let layers = match &client_config["layers"] {
        JsonValue::Array(names) => { Ok(names) }
        _ => { Err(TerminalError::InputConfigError) }
    }?.iter().map(|v| v.to_string()).collect::<Vec<_>>();

    if names.len() != layers.len() {
        return Err(TerminalError::BadArgError {
            arg: format!("names || layers"),
            msg: format!("count of names '{}' isn't equal to count of layers '{}'",
                         names.len(), layers.len()),
        });
    }

    let (split_names, split_map) =
        prepare_splits_info(names, layers)?;
    Ok((split_names, split_map))
}

fn parse_cfg_from_cli(sub: &ArgMatches)
                      -> Result<(Vec<String>, HashMap<String, i16>), TerminalError> {
    let names = sub
        .values_of("names")
        .ok_or_else(|| TerminalError::WithoutArgError {
            arg: format!("names"),
            msg: sub.usage().to_string(),
        })?
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    let layers = sub
        .values_of("layers")
        .ok_or_else(|| TerminalError::WithoutArgError {
            arg: format!("names"),
            msg: sub.usage().to_string(),
        })?
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    if names.len() != layers.len() {
        TerminalError::BadArgError {
            arg: format!("names || layers"),
            msg: format!("count of names '{}' isn't equal to count of layers '{}'",
                         names.len(), layers.len()),
        };
    }
    let (split_names, split_map) =
        prepare_splits_info(names, layers)?;
    Ok((split_names, split_map))
}

fn prepare_splits_info(names: Vec<String>, layers: Vec<String>)
                       -> Result<(Vec<String>, HashMap<String, i16>), TerminalError> {
    let mut split_names: Vec<String> = Vec::new();
    for name in names {
        if name.len() == 0 {
            return Err(TerminalError::BadArgError {
                arg: format!("names"),
                msg: format!("one of arg names NULL string"),
            });
        }
        split_names.push(name.to_string());
    }
    let mut split_map: HashMap<String, i16> = HashMap::new();
    let mut deduct_split = String::new();
    for (i, num) in layers.iter().enumerate() {
        let name = split_names
            .get(i)
            .unwrap();
        let value = num.parse::<i16>().unwrap();
        if deduct_split.len() == 0 && value == -1 {
            deduct_split = name.clone();
        } else if deduct_split.len() > 0 && value == -1 {
            return Err(TerminalError::BadArgError {
                arg: format!("layers"),
                msg: format!("more than 1 split of -1 layers: '{:?}' and '{:?}'",
                             deduct_split, name),
            });
        } else if value < 1 {
            return Err(TerminalError::BadArgError {
                arg: format!("layers"),
                msg: format!("split number can only be positive or -1, actuall '{:?}'", value),
            });
        }
        split_map.insert(name.clone(), value);
    }
    Ok((split_names, split_map))
}

pub fn cli_main(args: Vec<String>) -> Result<(), TerminalError> {
    let matches = App::new("LayerSword")
        .version("0.1.0")
        .author("cutecutecat")
        .about("Split or merge image files")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowMissingPositional)
        .subcommand(SubCommand::with_name("split")
            .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("FILE")
                .help("Pick [names && layers] settings from a custom config file"))
            .group(ArgGroup::with_name("from_file")
                .args(&["config"])
            )
            .arg(Arg::with_name("names")
                .short("n")
                .long("names")
                .takes_value(true)
                .value_name("STRING,STRING...")
                .use_delimiter(true)
                .required_unless("config")
                .conflicts_with("config")
                .requires("layers")
                .help("Names of the splits"))
            .arg(Arg::with_name("layers")
                .short("l")
                .long("layers")
                .takes_value(true)
                .value_name("INT,INT...")
                .use_delimiter(true)
                .required_unless("config")
                .conflicts_with("config")
                .requires("names")
                .help("Layer number of splits"))
            .arg(Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .value_name("FILE")
                .required(true)
                .help("Path of target image tar file"))
            .arg(Arg::with_name("work")
                .short("w")
                .long("work")
                .takes_value(true)
                .value_name("DIRECTORY")
                .default_value("tmp")
                .help("Path of temporary working directory"))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .value_name("DIRECTORY")
                .default_value("tmp")
                .help("Path of output directory"))
            .arg(Arg::with_name("silence")
                .short("s")
                .long("silence")
                .help("Not print anything to terminal"))
            .arg(Arg::with_name("level")
                .short("v")
                .long("level")
                .takes_value(true)
                .default_value("default")
                .value_name("INT[0-9]/NONE/FAST/DEFAULT/BEST")
                .possible_value("0")
                .possible_value("1")
                .possible_value("2")
                .possible_value("3")
                .possible_value("4")
                .possible_value("5")
                .possible_value("6")
                .possible_value("7")
                .possible_value("8")
                .possible_value("9")
                .possible_value("none")
                .possible_value("fast")
                .possible_value("default")
                .possible_value("best")
                .case_insensitive(true)
                .help("Compress level of tar.gz split file(0->none, 1->fast,...9->best)")))
        .subcommand(SubCommand::with_name("merge")
            .arg(Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .value_name("DIRECTORY")
                .required(true)
                .help("Path of target directory of tar.gz split files"))
            .arg(Arg::with_name("work")
                .short("w")
                .long("work")
                .takes_value(true)
                .value_name("DIRECTORY")
                .default_value("tmp")
                .help("Path of temporary working directory")).
            arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .value_name("DIRECTORY")
                .default_value("tmp")
                .help("Path of output directory"))
            .arg(Arg::with_name("silence")
                .short("s")
                .long("silence")
                .help("Not print anything to terminal"))
        ).get_matches_from_safe(args)?;

    if !matches.is_present("silence") {
    }
    if let Some(sub) = matches.subcommand_matches("split") {
        let (target_path, work_path, out_path) =
            parse_path(&sub, "split")?;
        let level_str = sub.value_of("level")
            .ok_or_else(|| TerminalError::WithoutArgError {
                arg: format!("level"),
                msg: sub.usage().to_string(),
            })?;
        let level_map: HashMap<&str, u8> = [("none", 0), ("fast", 1), ("default", 6), ("best", 9)]
            .iter().cloned().collect();
        let level_from_map = level_map.get(level_str);
        let level_from_conv = level_str.parse::<u8>();
        let mut level: u8 = 6;
        if level_from_map.is_some() {
            level = *level_from_map.unwrap();
        } else if level_from_conv.is_ok() {
            level = level_from_conv.unwrap();
        }

        if sub.is_present("config") {
            let (split_names, split_map) = parse_cfg_from_file(sub)?;
            init_path(work_path);
            // TODO:测试下这里能不能unwrap
            split_layer(target_path, split_names, split_map,
                        work_path, out_path, level).unwrap();
        } else if sub.is_present("names") & sub.is_present("layers") {
            let (split_names, split_map) = parse_cfg_from_cli(sub)?;
            init_path(work_path);
            split_layer(target_path, split_names, split_map,
                        work_path, out_path, level).unwrap();
        } else {
            TerminalError::WithoutArgError {
                arg: format!("(names && layers) || config"),
                msg: matches.usage().to_string(),
            };
        }
    } else if let Some(sub) = matches.subcommand_matches("merge") {
        let (target_path, work_path, out_path) =
            parse_path(&sub, "merge")?;
        init_path(work_path);
        merge_layer(target_path, work_path, out_path).unwrap();
    }
    Ok(())
}