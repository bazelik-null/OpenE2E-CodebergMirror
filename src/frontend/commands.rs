use log::error;

pub enum Command {
    // Misc
    Exit, // exit
    // User control
    NewUser,    // u new
    DeleteUser, // u delete
    ExitUser,   // u exit
    OpenUser,   // u open
    ListUsers,  // u list
    // Session control
    NewSession,    // s new
    DeleteSession, // s delete
    ExitSession,   // s exit
    OpenSession,   // s open
    ListSessions,  // s list
    // Message control
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
        "u" => scan_user_commands(argument),
        _ => {
            error!("Unknown command: {}", argument);
            None
        }
    }
}

fn scan_session_commands(argument: &str) -> Option<Command> {
    match argument {
        "new" => Some(Command::NewSession),
        "delete" => Some(Command::DeleteSession),
        "exit" => Some(Command::ExitSession),
        "open" => Some(Command::OpenSession),
        "list" => Some(Command::ListSessions),
        _ => {
            error!("Unknown argument: {}", argument);
            None
        }
    }
}

fn scan_user_commands(argument: &str) -> Option<Command> {
    match argument {
        "new" => Some(Command::NewUser),
        "delete" => Some(Command::DeleteUser),
        "exit" => Some(Command::ExitUser),
        "open" => Some(Command::OpenUser),
        "list" => Some(Command::ListUsers),
        _ => {
            error!("Unknown argument: {}", argument);
            None
        }
    }
}
