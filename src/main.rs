use std::fs::File;
use std::io::Read;

fn main() {
    let app = clap::App::new("amethyst_parser_syn").subcommand(
        clap::SubCommand::with_name("implementor")
            .help("Find types that implement a specific trait")
            .arg(
                clap::Arg::with_name("trait")
                    .help("Trait to find implementors for")
                    .short("i")
                    .long("impls")
                    .takes_value(true)
                    .required(true),
            )
            .arg(clap::Arg::with_name("FILES").required(true).multiple(true)),
    );

    match app.get_matches().subcommand() {
        ("implementor", Some(matches)) => {
            let files = matches
                .values_of("FILES")
                .map(|files| files.map(|it| String::from(it)).collect::<Vec<String>>())
                .unwrap_or(Vec::new());

            let impls = matches.value_of("trait").expect("impls was not specified");

            for filename in files {
                let mut file = File::open(&filename).expect("Unable to open file");
                let mut src = String::new();
                file.read_to_string(&mut src).expect("Unable to read file");

                let structs = amethyst_parser_syn::FindImplementorsVisiter::file(
                    &src,
                    impls,
                );

                if let Ok(found) = structs {
                    if found.len() != 0 {
                        println!("File: {}", filename);
                    }
                    for struct_ in found {
                        println!("Struct: {}", struct_.struct_name);
                        if let Some(doc) = struct_.struct_doc {
                            println!("{}", doc);
                        }
                        println!();
                    }
                }
            }
        }
        _ => {}
    }
}
