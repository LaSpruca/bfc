use std::{
    fs::{read_to_string, File},
    io::{BufWriter, Write},
    process::{exit, Command},
};

use env_logger::Env;
use log::{error, info};

fn el_comprec(src: &str) -> Vec<(String, i32)> {
    let mut current_token = "";
    let mut count = 0;
    let mut comprec_code = vec![];

    for cha in src.split("") {
        if cha == current_token {
            count += 1;
        } else {
            // Only add the current token and it's count to the array if it is a valid BrainFuck token
            if current_token != "" {
                comprec_code.push((current_token.to_owned(), count));
            }

            match cha {
                // If the character is a BrainFuck token
                "." | "," | "+" | "-" | "<" | ">" | "[" | "]" => {
                    current_token = cha;
                    count = 1;
                }
                // If the character is not a BrainFuck token
                _ => current_token = "",
            };
        }
    }

    comprec_code
}

fn init_logger() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();
}

fn main() {
    init_logger();
    let args = std::env::args()
        .skip(1)
        .map(|e| e.to_string())
        .collect::<Vec<String>>();

    if args.is_empty() {
        error!("Please provide some args");
        exit(-1);
    }

    info!("Files to compile: {:?}", args);

    for file in args {
        let intermediate = file.replace(".bf", ".rs");
        let src = match read_to_string(file) {
            Ok(source_code) => source_code,
            Err(e) => {
                eprintln!("Error reading file\n{}", e);
                continue;
            }
        };

        // Compress the source code into a list of characters and the amount of times that they occur
        let compressed = el_comprec(&src);

        info!("{:?}", compressed);

        //     let mut code = r#"use std::io::{stdin,stdout,prelude::*};fn i(m:&mut[i32;30000],p:&usize){loop{print!(">>> ");stdout().flush().unwrap();let mut temp="".to_string();stdin().read_line(&mut temp).unwrap();println!();if let Ok(v)=temp.trim().parse::<i32>(){m[p.to_owned()]=v;return;}else{println!("{} is not a number",temp);}}}fn main(){let mut m=[0;30000];let mut p=0usize;"#
        //     .to_string();

        //     code += "}";

        //     match File::create(intermediate.clone()) {
        //         Ok(file) => {
        //             let mut writer = BufWriter::new(&file);
        //             match write!(&mut writer, "{}", code) {
        //                 Ok(_) => {}
        //                 Err(e) => {
        //                     eprintln!("Error reading file\n{}", e);
        //                     continue;
        //                 }
        //             }
        //         }
        //         Err(e) => {
        //             eprintln!("Error creating file file\n{}", e);
        //             continue;
        //         }
        //     }

        //     println!(
        //         "{:?}",
        //         Command::new("rustc")
        //             .args(&[&intermediate, "-C", "debuginfo=0", "-C", "opt-level=3"])
        //             .output()
        //             .expect("Error compiling rust code")
        //     );

        //     match std::fs::remove_file(intermediate) {
        //         Ok(_) => {}
        //         Err(e) => {
        //             eprintln!("Error removing intermediate file file\n{}", e);
        //             continue;
        //         }
        //     };
    }
}
