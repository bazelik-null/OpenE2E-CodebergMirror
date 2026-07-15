use log::error;

pub enum Command {
    // Misc
    Exit, // exit
    Help, // help

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
    Encrypt, // e
    Decrypt, // d
}

pub fn scan_commands(input: &str) -> Option<Command> {
    // Split input to command and arguments
    let mut input_it = input.split_whitespace();
    let command = input_it.next().unwrap_or("");
    let subcommand = input_it.next().unwrap_or("");
    // todo: arguments
    // let arguments: Vec<&str> = input_it.collect();

    match command {
        "exit" => Some(Command::Exit),
        "help" => Some(Command::Help),
        "e" => Some(Command::Encrypt),
        "d" => Some(Command::Decrypt),
        "s" => scan_session_commands(subcommand),
        "u" => scan_user_commands(subcommand),
        _ => {
            error!("Unknown command: '{}'", command);
            None
        }
    }
}

fn scan_session_commands(subcommand: &str) -> Option<Command> {
    match subcommand {
        "new" => Some(Command::NewSession),
        "delete" => Some(Command::DeleteSession),
        "exit" => Some(Command::ExitSession),
        "open" => Some(Command::OpenSession),
        "list" => Some(Command::ListSessions),
        _ => {
            error!("Unknown session command: '{}'", subcommand);
            None
        }
    }
}

fn scan_user_commands(subcommand: &str) -> Option<Command> {
    match subcommand {
        "new" => Some(Command::NewUser),
        "delete" => Some(Command::DeleteUser),
        "exit" => Some(Command::ExitUser),
        "open" => Some(Command::OpenUser),
        "list" => Some(Command::ListUsers),
        _ => {
            error!("Unknown user command: '{}'", subcommand);
            None
        }
    }
}
