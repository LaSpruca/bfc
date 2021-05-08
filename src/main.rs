use std::{
    fs::{read_to_string, File},
    io::{BufWriter, Write},
    process::Command,
};

fn main() {
    let args = std::env::args()
        .skip(1)
        .map(|e| e.to_string())
        .collect::<Vec<String>>();

    if args.is_empty() {
        println!("Please provide some args");
    }

    println!("Files to compile: {:?}", args);

    for file in args {
        let intermediate = file.replace(".bf", ".rs");
        let src = match read_to_string(file) {
            Ok(source_code) => source_code,
            Err(e) => {
                eprintln!("Error reading file\n{}", e);
                continue;
            }
        };
        let mut code = r#"use std::io::{stdin,stdout,prelude::*};fn i(m:&mut[i32;30000],p:&usize){loop{print!(">>> ");stdout().flush().unwrap();let mut temp="".to_string();stdin().read_line(&mut temp).unwrap();println!();if let Ok(v)=temp.trim().parse::<i32>(){m[p.to_owned()]=v;return;}else{println!("{} is not a number",temp);}}}fn main(){let mut m=[0;30000];let mut p=0usize;"#
        .to_string();

        for cha in src.split("") {
            code += match cha {
                "." => "println!(\"{}\",m[p]);",
                "," => "i(&mut m, &p);",
                "+" => "m[p]+=1;",
                "-" => "m[p]-=1;",
                "<" => "p-=1;if p > 30000 {p=29999}",
                ">" => "p+=1;if p > 29999 {p=0}",
                "[" => "while m[p] != 0 {",
                "]" => "}",
                _ => "",
            }
        }
        code += "}";

        match File::create(intermediate.clone()) {
            Ok(file) => {
                let mut writer = BufWriter::new(&file);
                match write!(&mut writer, "{}", code) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error reading file\n{}", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error creating file file\n{}", e);
                continue;
            }
        }

        println!(
            "{:?}",
            Command::new("rustc")
                .args(&[&intermediate, "-C", "debuginfo=0", "-C", "opt-level=3"])
                .output()
                .expect("Error compiling rust code")
        );

        match std::fs::remove_file(intermediate) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error removing intermediate file file\n{}", e);
                continue;
            }
        };
    }
}
