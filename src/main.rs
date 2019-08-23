extern crate rustyline;

use rust_forth_compiler::ForthCompiler;
use rust_forth_compiler::ForthError;
use rust_forth_compiler::GasLimit;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

/// This Enum lists the errors that the Forth Interpreter might return
#[derive(Debug)]
pub enum ForthInteractiveError {
    UnknownError,
    ForthError(ForthError),
    IOError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
}

pub enum CommandHandled {
    Handled,
    NotHandled,
}

// Chain of Command Pattern
pub trait HandleCommand {
    fn handle_command(
        &mut self,
        command_id: &str,
        parameters: &[&str],
        fc: &mut ForthCompiler,
    ) -> Result<CommandHandled, ForthInteractiveError>;
    fn command_id(&self) -> String;
    fn help_text(&self) -> String;
}

pub struct CommandHandler<'a> {
    command_id: String,
    help_text: String,
    to_run: Box<
        dyn Fn(&str, &[&str], &mut ForthCompiler) -> Result<CommandHandled, ForthInteractiveError>
            + 'a,
    >,
}

impl<'a> CommandHandler<'a> {
    pub fn new<C>(command_id: &str, help_text: &str, f: C) -> CommandHandler<'a>
    where
        C: Fn(&str, &[&str], &mut ForthCompiler) -> Result<CommandHandled, ForthInteractiveError>
            + 'a,
    {
        CommandHandler {
            command_id: command_id.to_owned(),
            help_text: help_text.to_owned(),
            to_run: Box::new(f),
        }
    }
}

impl<'a> HandleCommand for CommandHandler<'a> {
    fn handle_command(
        &mut self,
        command_id: &str,
        parameters: &[&str],
        fc: &mut ForthCompiler,
    ) -> Result<CommandHandled, ForthInteractiveError> {
        if command_id == self.command_id {
            return (self.to_run)(self.command_id.as_ref(), parameters, fc);
        }
        Ok(CommandHandled::NotHandled)
    }

    fn command_id(&self) -> String {
        self.command_id.clone()
    }

    fn help_text(&self) -> String {
        self.help_text.clone()
    }
}

/// Convert std::num::ParseIntError to a ForthInteractiveError so our functions can
/// return a single Error type.
impl From<std::num::ParseIntError> for ForthInteractiveError {
    fn from(err: std::num::ParseIntError) -> ForthInteractiveError {
        ForthInteractiveError::ParseIntError(err)
    }
}

/// Convert std::num::ParseIntError to a ForthInteractiveError so our functions can
/// return a single Error type.
impl From<ForthError> for ForthInteractiveError {
    fn from(err: ForthError) -> ForthInteractiveError {
        ForthInteractiveError::ForthError(err)
    }
}

/// Convert std::io::Error to a ForthInteractiveError so our functions can
/// return a single Error type.
impl From<std::io::Error> for ForthInteractiveError {
    fn from(err: std::io::Error) -> ForthInteractiveError {
        ForthInteractiveError::IOError(err)
    }
}

fn main() -> Result<(), ForthError> {
    println!("This is the rust-forth-interactive-compiler");

    let mut fc = ForthCompiler::new();

    let mut command_handlers: Vec<Box<dyn HandleCommand>> = Vec::new();
    command_handlers.push(Box::from(CommandHandler::new(
        "l",
        "Load Forth file: file1.f [file2.f]",
        |_command_id, params, fc| {
            for n in params {
                let startup = fs::read_to_string(n)?;
                fc.execute_string(&startup, GasLimit::Limited(100))?;
            }
            Ok(CommandHandled::Handled)
        },
    )));

    command_handlers.push(Box::from(CommandHandler::new(
        "n",
        "Print number stack: No Parameters",
        |_command_id, _params, fc| {
            println!("Number Stack {:?}", fc.sm.st.number_stack);
            Ok(CommandHandled::Handled)
        },
    )));

    command_handlers.push(Box::from(CommandHandler::new(
        "p",
        "Push numbers on stack: n1 [n2]",
        |_command_id, params, fc| {
            for n in params {
                fc.sm.st.number_stack.push(n.parse::<i64>()?);
            }
            Ok(CommandHandled::Handled)
        },
    )));

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

                // Okay, so we have a line, each line starts with a command, and then has optional parameters
                let words: Vec<&str> = line.split_whitespace().collect();
                // If nothing to talk about, just ignore...
                if words.len() == 0 {
                    continue;
                }

                let command = words[0];
                let parameters = &words[1..];

                // Try to handle the command here
                let mut handled = false;
                for h in command_handlers.iter_mut() {
                    if let Ok(CommandHandled::Handled) =
                        h.handle_command(command, parameters, &mut fc)
                    {
                        handled = true;
                    }
                }

                if !handled {
                    println!("Help text:");
                    for h in command_handlers.iter() {
                        println!("    Command: {} Usage: {}", h.command_id(), h.help_text());
                    }
                }
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
