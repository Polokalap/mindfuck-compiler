use ceasa::randint;
use std::{env, process};
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::{exit, Command};
use std::time::Instant;

//
// MindFuck made by: Polokalap in 2025 on Arch Linux. I like femboys
//

enum CommandType {
    PRINT,
    READ,
    COMMENT,
}

fn check_syntax(line: String, list: &mut Vec<String>, cod: &mut Vec<String>, dec: &mut Vec<String>, debug: bool) {
    let read_line = line;
    let command: CommandType;

    if read_line.starts_with("> ") {
        command = CommandType::PRINT;
    } else if read_line.starts_with("< ") {
        command = CommandType::READ;
    } else {
        command = CommandType::COMMENT;
    }

    let _ = make_asm_code(
        command,
        &mut read_line.replace("> ", "").replace("< ", ""),
        list,
        dec,
        cod,
        debug,
    );
}

fn make_asm_code(
    command: CommandType,
    command_line: &mut String,
    list: &mut Vec<String>,
    cod: &mut Vec<String>,
    dec: &mut Vec<String>,
    debug: bool,
) -> io::Result<()> {
    match command {
        CommandType::PRINT => {
            list.push(command_line.to_string());

            let id = randint(0, i32::MAX);

            dec.push(format!(
                "msg{id} db '{command_line}', 0xA\nlen{id} equ $ - msg{id}"
            ));
            cod.push(format!(
                "mov rax, 1\nmov rdi, 1\nmov rsi, msg{id}\nmov rdx, len{id}\nsyscall"
            ));
        }
        CommandType::READ => {
            list.push(command_line.to_string());

            let id = randint(0, i32::MAX);

            if !command_line.is_empty() {
                dec.push(format!(
                    "msg{id} db '{command_line}', 0xA\nlen{id} equ $ - msg{id}"
                ));
                cod.push(format!(
                    "mov rax, 1\nmov rdi, 1\nmov rsi, msg{id}\nmov rdx, len{id}\nsyscall"
                ));
            }

            dec.push(format!("input{id} resb 64"));
            cod.push(format!(
                "mov rax, 0\nmov rdi, 0\nmov rsi, input{id}\nmov rdx, 64\nsyscall"
            ));
        }
        CommandType::COMMENT => {
            if debug {
                println!("Compiler ignoring comment: {}", command_line);
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {

    let timer = Instant::now();

    let args: Vec<String> = env::args().collect();
    let debug = args.contains(&"-d".to_string()) || args.contains(&"--debug".to_string());

    let file = match File::open("main.mf") {

        Ok(file) => file,
        Err(_) => {

            eprintln!("Couldn't find main.mf");
            std::process::exit(1);

        }

    };

    let reader = BufReader::new(file);


    let mut list: Vec<String> = vec![];
    let mut dec: Vec<String> = vec![];
    let mut cod: Vec<String> = vec![];

    for line in reader.lines() {
        let line = &line?;

        check_syntax(line.to_string(), &mut list, &mut dec, &mut cod, debug);

        if debug {
            println!("{}", line);
        }
    }

    let template = "\
    section .data
        ; >>dec<<
    section .text
        global _start
    _start:
        ; >>cod<<
        mov rax, 60           ;
        xor rdi, rdi          ;
        syscall               ;
    ";

    let compiled = template
        .replace("; >>dec<<", &format!("{}\n    ; >>dec<<", dec.join("\n")))
        .replace("; >>cod<<", &format!("{}\n    ; >>cod<<", cod.join("\n")));

    if debug {
        println!("{}", compiled);
    }

    let _ = fs::create_dir("target");
    let _ = fs::create_dir("target/temp");
    fs::write("target/temp/wfc.pef", compiled)?;

    let compiled_file_path = "target/compiled";

    Command::new("nasm")
        .args(&[
            "-f",
            "elf64",
            "-w-zeroing",
            "target/temp/wfc.pef",
            "-o",
            "target/temp/wfc.t",
        ])
        .status()?;
    Command::new("ld")
        .args(&["target/temp/wfc.t", "-o", compiled_file_path])
        .status()?;


    let duration = timer.elapsed();

    Ok(println!("Compiled succesfully! Took: {:?}", duration))

}
