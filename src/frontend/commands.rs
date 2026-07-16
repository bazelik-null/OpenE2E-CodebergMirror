pub enum Command {
    // Misc
    Exit,
    Help,

    // User control
    NewUser { name: String, password: String },
    DeleteUser { name: String },
    LogoutUser,
    LoginUser { name: String },
    ListUsers,

    // Session control
    NewSession { name: String },
    DeleteSession { name: String },
    ExitSession,
    OpenSession { name: String },
    ListSessions,

    // Message control
    Encrypt { text: String },
    Decrypt { text: String },
}

pub fn scan_commands(input: &str) -> Option<Command> {
    let mut input_it = input.split_whitespace();
    let command = input_it.next().unwrap_or("");
    let subcommand = input_it.next().unwrap_or("");
    let arguments: Vec<&str> = input_it.collect();

    match command {
        "exit" => Some(Command::Exit),
        "help" => Some(Command::Help),
        "e" => {
            if !subcommand.is_empty() {
                Some(Command::Encrypt {
                    text: subcommand.to_string(),
                })
            } else {
                None
            }
        }
        "d" => {
            if !subcommand.is_empty() {
                Some(Command::Decrypt {
                    text: subcommand.to_string(),
                })
            } else {
                None
            }
        }
        "s" => scan_session_commands(subcommand, &arguments),
        "u" => scan_user_commands(subcommand, &arguments),
        _ => None,
    }
}

fn scan_session_commands(subcommand: &str, arguments: &[&str]) -> Option<Command> {
    match subcommand {
        "new" => {
            if !arguments.is_empty() {
                Some(Command::NewSession {
                    name: arguments[0].to_string(),
                })
            } else {
                None
            }
        }
        "delete" => {
            if !arguments.is_empty() {
                Some(Command::DeleteSession {
                    name: arguments[0].to_string(),
                })
            } else {
                None
            }
        }
        "exit" => Some(Command::ExitSession),
        "open" => {
            if !arguments.is_empty() {
                Some(Command::OpenSession {
                    name: arguments[0].to_string(),
                })
            } else {
                None
            }
        }
        "list" => Some(Command::ListSessions),
        _ => None,
    }
}

fn scan_user_commands(subcommand: &str, arguments: &[&str]) -> Option<Command> {
    match subcommand {
        "new" => {
            if arguments.len() >= 2 {
                Some(Command::NewUser {
                    name: arguments[0].to_string(),
                    password: arguments[1].to_string(),
                })
            } else {
                None
            }
        }
        "delete" => {
            if !arguments.is_empty() {
                Some(Command::DeleteUser {
                    name: arguments[0].to_string(),
                })
            } else {
                None
            }
        }
        "logout" => Some(Command::LogoutUser),
        "login" => {
            if !arguments.is_empty() {
                Some(Command::LoginUser {
                    name: arguments[0].to_string(),
                })
            } else {
                None
            }
        }
        "list" => Some(Command::ListUsers),
        _ => None,
    }
}
