use std::{
    fs::{read_to_string, File},
    io::{BufWriter, Write},
    os::unix::prelude::ExitStatusExt,
    process::{exit, Command, ExitStatus},
};

use colored::*;
use log::{debug, error, info};

/// Take raw source code and reduce it to a vector of BrainFuck tokens and the number of times that the recursively appear
fn reduce(src: &str) -> Vec<(String, i32)> {
    let mut current_token = "";
    let mut count = 0;
    let mut reduced = vec![];

    for cha in src.split("") {
        if cha == current_token {
            count += 1;
        } else {
            // Only add the current token and it's count to the array if it is a valid BrainFuck token
            if current_token != "" {
                reduced.push((current_token.to_owned(), count));
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

    reduced
}

/// Function to compile a file
fn compile_file(src: &String) -> (String, bool) {
    // Reduce the source code into a list of characters and the amount of times that they occur
    let reduced = reduce(&src);

    debug!("{:?}", reduced);

    // Setup the default application
    let mut code = String::new();

    // Flag if the code requires inputs from the user
    let mut has_input = false;

    // Loop through all of the tokens
    for (token, count) in reduced.iter() {
        // Generate the rust code from the tokens
        match token.as_str() {
            // Print the current value
            "." => {
                code += r#"println!("{}",m[p]);"#;
            }
            // Get input from the user
            "," => {
                code += r#"i(&mut m,&p);"#;
                // Set the has_input flag to true
                has_input = true;
            }
            // Increment the current memory cell
            "+" => {
                code += format!(r#"m[p]+={};"#, count).as_str();
            }
            // Decrement the current memory cell
            "-" => {
                code += format!(r#"m[p]-={};"#, count).as_str();
            }
            // Loop while the current cell is not 0
            "[" => {
                for _ in 0..count.to_owned() {
                    code += "while m[p]!=0{";
                }
            }
            // Close loops
            "]" => {
                for _ in 0..count.to_owned() {
                    code += "}";
                }
            }
            // Shift the pointer to the left, if overflow, rest to 2999, and subtract overflow from pointer
            "<" => {
                code += format!(r#"{{let i=p as i32-{};if i<0{{p=(2999-(i+1).abs()) as usize;}}else{{p=i as usize;}}}}"#, count).as_str();
            }
            // Shift the pointer to the right, if overflow, rest to 0, and add overflow to pointer
            ">" => {
                code += format!(
                            r#"{{let i=p as i32+{};if i>2999{{p=((i-1-2999).abs())as usize;}}else{{p=i as usize;}}}}"#,
                            count
                        )
                        .as_str();
            }
            _ => {}
        }
    }

    (code, has_input)
}

/// Function to initialize the logging system
fn init_logger() {
    // Set the log level environment variable if not already set.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[ {} ] {}",
                // Color the log levels
                match record.level() {
                    log::Level::Error => {
                        "  ERROR  ".red().bold()
                    }
                    log::Level::Warn => {
                        " WARNING ".yellow().bold()
                    }
                    log::Level::Info => {
                        "   INFO  ".blue().bold()
                    }
                    log::Level::Debug => {
                        "  DEBUG  ".white().bold()
                    }
                    log::Level::Trace => {
                        "  TRACE  ".black().bold()
                    }
                },
                record.args()
            )
        })
        .init();
}

fn main() {
    init_logger();
    let mut args = std::env::args().skip(1).collect::<Vec<String>>();

    let delete_intermediate = !args.contains(&("-k".to_string()));

    args = args
        .iter()
        .filter(|e| e.as_str() != "-k")
        .map(|e| e.to_string())
        .collect();

    if args.is_empty() {
        error!("Please provide some args");
        exit(-1);
    }

    info!("Files to compile: {:?}", args.join(", "));

    // Location to store the intermediate rust source code
    let mut intermediate = args[0].clone().replace(".bf", ".rs");
    if !intermediate.ends_with(".rs") {
        intermediate += ".rs";
    }

    // The source code of all the compiled files
    let mut code = r#"fn main(){let mut m=[0;30000];let mut p=0usize;"#.to_string();
    let mut has_input = false;

    for file in args {
        // Generate the file name for the intermediate file

        // Read the source code
        let src = match read_to_string(file) {
            Ok(source_code) => source_code,
            Err(e) => {
                eprintln!("Error reading file\n{}", e);
                continue;
            }
        };

        let (file_code, file_has_input) = compile_file(&src);

        code += file_code.as_str();
        has_input = has_input || file_has_input;
    }

    // Close the main function and add the input function if the code actually uses it
    if has_input {
        code += r#"}use std::io::{stdin,stdout,prelude::*};fn i(m:&mut[i32;30000],p:&usize){loop{print!(">>> ");stdout().flush().unwrap();let mut temp="".to_string();stdin().read_line(&mut temp).unwrap();println!();if let Ok(v)=temp.trim().parse::<i32>(){m[p.to_owned()]=v;return;}else{println!("{} is not a number",temp);}}}"#;

    // Otherwise just close the main function
    } else {
        code += "}";
    }

    info!("Compiling with rustc");

    // Write the code to an intermediate file
    match File::create(intermediate.clone()) {
        Ok(file) => {
            let mut writer = BufWriter::new(&file);
            match write!(&mut writer, "{}", code) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading file\n{}", e);
                    exit(-1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating file file\n{}", e);
            exit(-1);
        }
    }

    // Compile with rustc

    let output = Command::new("rustc")
        .args(&[
            &intermediate.as_str(),
            "-C",
            "debuginfo=0",
            "-C",
            "opt-level=3",
        ])
        .output()
        .expect("Error compiling rust code, is rustc installed and in path?");

    if output.status != ExitStatus::from(ExitStatusExt::from_raw(0x0)) {
        error!(
            "Error compiling rust code, check your braces. Run with -k to keep the intermediate"
        );
    }

    if delete_intermediate {
        // Delete the intermediate file
        match std::fs::remove_file(intermediate) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error removing intermediate file file\n{}", e);
                exit(-1);
            }
        };
    }
}
