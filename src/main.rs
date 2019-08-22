extern crate rustyline;

use rust_forth_compiler::ForthCompiler;
use rust_forth_compiler::ForthError;
use rust_forth_compiler::GasLimit;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

fn main() -> Result<(), ForthError> {
    println!("This is the rust-forth-interactive-compiler");

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();

    run()?;

    Ok(())
}

fn run() -> Result<(), ForthError> {
    let mut fc = ForthCompiler::new();

    //fc.execute_string("1 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))?;
    fc.execute_string("0 IF 1 2 ADD THEN", GasLimit::Limited(100))?;

    println!("Contents of Number Stack {:?}", fc.sm.st.number_stack);
    Ok(())
}
