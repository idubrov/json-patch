use serde_json::{de::IoRead, Deserializer, Value};
use std::{convert::TryInto, env::args_os, fs::File, io::stdin, process::exit};

fn usage(header: bool) -> ! {
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_ver = env!("CARGO_PKG_VERSION");
    let exe_path = args_os().next();
    let exe_path = exe_path
        .as_deref()
        .and_then(|path| path.to_str())
        .unwrap_or(pkg_name);
    if header {
        eprintln!("{} {}", pkg_name, pkg_ver);
        eprintln!("RFC 6902 JSON patch calculation tool");
    }
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    {} diff ./original.json ./changed.json", exe_path);
    eprintln!("    {} merge ./original.json ./patch.json", exe_path);
    eprintln!();
    eprintln!("    A dash (-) can be used to read a json object from stdin.");
    eprintln!("    When both files are dashes, the original is expected first.");
    exit(-1);
}

enum Op {
    Diff,
    Merge,
}

fn main() {
    let argv = args_os().collect::<Vec<_>>();
    let (op, original, a2) = match &argv[..] {
        [] => usage(true),
        [_, op, original, a2] => (op, original, a2),
        _ => {
            eprintln!("Wrong number of arguments, expecting precisely three!");
            usage(false)
        }
    };
    let op = match op.to_str() {
        Some("diff") => Op::Diff,
        Some("merge") => Op::Merge,
        op => {
            eprintln!("Unknown operation: {}", op.unwrap_or("[UTF-8 invalid]"));
            usage(false);
        }
    };
    let parsed = if original.to_str() == Some("-") && a2.to_str() == Some("-") {
        let input = Deserializer::new(IoRead::new(stdin()))
            .into_iter()
            .collect::<Result<Vec<Value>, _>>();
        match input {
            Err(e) => {
                eprintln!("Could not parse stdin: {}", e);
                exit(-1);
            }
            Ok(input) if input.len() != 2 => {
                eprintln!("Expected precisely two JSON objects on stdin");
                exit(-1);
            }
            Ok(input) => input,
        }
    } else {
        [original, a2]
            .iter()
            .map(|path| {
                let parse = if path.to_str() == Some("-") {
                    serde_json::from_reader(stdin())
                } else {
                    let file = match File::open(path) {
                        Ok(file) => file,
                        Err(e) => {
                            eprintln!("Could not open {}: {}", path.to_string_lossy(), e);
                            exit(-1);
                        }
                    };
                    serde_json::from_reader(file)
                };
                match parse {
                    Ok(parse) => parse,
                    Err(e) => {
                        eprintln!("Could not read JSON from {}: {}", path.to_string_lossy(), e);
                        exit(-1);
                    }
                }
            })
            .collect()
    };
    let [original, a2]: [Value; 2] = parsed.try_into().expect("Length 2 was checked");

    match op {
        Op::Diff => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json_patch::diff(&original, &a2))
                    .expect("Serializing a JSON Patch to JSON should not fail")
            );
        }
        Op::Merge => {
            let patch = match json_patch::from_value(a2) {
                Ok(patch) => patch,
                Err(e) => {
                    eprintln!("Third argument is not a valid JSON patch: {}", e);
                    exit(2);
                }
            };
            let mut patchable = original;
            if let Err(e) = json_patch::patch(&mut patchable, &patch) {
                eprintln!("Patch did not apply: {}", e);
                exit(1);
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&patchable)
                    .expect("Serializing a JSON Value to JSON should not fail")
            );
        }
    };
}
