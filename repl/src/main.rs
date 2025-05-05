use rustyline::Editor;
use rustyline::error::ReadlineError;

use bumpalo::Bump;
use qparser::eval_str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = Editor::<()>::new()?;
    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(input) => {
                rl.add_history_entry(input.as_str());
                let _bump = Bump::new();
                match eval_str(&input) {
                    Ok(val) => println!("=> {}", val),
                    Err(err) => eprintln!("Error: {}", err),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting.");
                break;
            }
            Err(err) => {
                eprintln!("Error reading line: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
