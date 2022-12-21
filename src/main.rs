use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use clap::Parser;
use codegen::Scope;
use convert_case::{Case, Casing};
use validator::{Validate, ValidationError};
use yaml_rust::{Yaml, YamlLoader};

/// Genarate Router for Axum
#[derive(Parser, Debug, Validate)]
#[command(author, version, about)]
struct Args {
    /// Path of OpenAPI spec yaml
    #[validate(length(min = 1), custom = "validate_file_path")]
    #[arg(short, long, required(true))]
    spec: String,

    /// Path of output file
    #[arg(short, long, required(false), default_value("router.rs"))]
    output: String,
}

fn main() {
    let args = Args::parse();
    match args.validate() {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    generate(args);
}

fn validate_file_path(spec: &String) -> Result<(), ValidationError> {
    match std::path::Path::new(Path::new(spec)).is_file() {
        true => Ok(()),
        false => Err(ValidationError::new("is not file path")),
    }
}

fn load_yaml(path: &str) -> Vec<Yaml> {
    let f = fs::read_to_string(path);
    let s = f.unwrap().to_owned();
    let docs = YamlLoader::load_from_str(&s).unwrap();
    docs
}

fn generate(args: Args) {
    let mut scope = Scope::new();
    let mut line: String = "".to_owned();

    let docs = load_yaml(&args.spec.as_str());
    let doc = docs.first().unwrap();

    for path in doc["paths"].as_hash().unwrap() {
        let mut methods: String = "".to_owned();

        for method in path.1.as_hash().unwrap() {
            if methods.len() == 0 {
                methods = format!(
                    "{}(handler::{})",
                    method.0.as_str().unwrap(),
                    method.1["operationId"]
                        .as_str()
                        .unwrap()
                        .to_case(Case::Snake)
                );
            } else {
                methods = format!(
                    "{}.{}(handler::{})",
                    methods,
                    method.0.as_str().unwrap(),
                    method.1["operationId"]
                        .as_str()
                        .unwrap()
                        .to_case(Case::Snake)
                );
            }
        }

        line = format!(
            "{}.route(\"{}\", {})",
            line,
            path.0.as_str().unwrap(),
            methods
        );
    }

    scope
        .new_fn("router")
        .vis("pub")
        .ret("axum::Router")
        .line(format!("axum::Router::new(){}", line));

    let mut file = File::create(args.output).unwrap();
    match write!(file, "{}", scope.to_string()) {
        Ok(_) => match file.flush() {
            Ok(_) => (),
            Err(e) => println!("{:?}", e),
        },
        Err(e) => println!("{:?}", e),
    }
}
