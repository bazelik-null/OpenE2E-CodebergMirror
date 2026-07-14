use log::error;

pub enum Command {
    Exit,            // exit
    NewSession,      // s new
    ExitSession,     // s exit
    OpenSession,     // s open
    Encrypt(String), // e {msg}
    Decrypt(String), // d {msg}
}

pub fn scan_commands(input: &str) -> Option<Command> {
    // Split input to command and argument by first whitespace
    let mut input_it = input.splitn(2, char::is_whitespace);
    let command = input_it.next().unwrap_or("");
    let argument = input_it.next().unwrap_or("");

    match command {
        "exit" => Some(Command::Exit),
        "e" => Some(Command::Encrypt(argument.to_string())),
        "d" => Some(Command::Decrypt(argument.to_string())),
        "s" => scan_session_commands(argument),
        _ => {
            error!("Unknown command: {}", argument);
            None
        }
    }
}

fn scan_session_commands(argument: &str) -> Option<Command> {
    match argument {
        "new" => Some(Command::NewSession),
        "exit" => Some(Command::ExitSession),
        "open" => Some(Command::OpenSession),
        _ => {
            error!("Unknown argument: {}", argument);
            None
        }
    }
}
