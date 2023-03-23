use colored::Colorize;
use inquire::{min_length, Password, PasswordDisplayMode};
use url::Url;

use envio::{
    self, check_profile, create_profile, delete_profile, download_profile, get_profile,
    import_profile, list_profiles,
};

use crate::cli::Command;

fn get_userkey() -> String {
    println!("{}", "Loading Profile".green());
    println!("{}", "Enter your encryption key".green());
    let prompt = Password::new("Enter your encryption key:")
        .with_display_toggle_enabled()
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_help_message("OH NO! you forgot your key! just kidding... or did you?")
        .without_confirmation()
        .prompt();

    if let Err(e) = prompt {
        println!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    } else {
        prompt.unwrap()
    }
}

impl Command {
    pub fn run(&self) {
        match self {
            Command::Create(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let prompt = Password::new("Enter your encryption key:")
                    .with_display_toggle_enabled()
                    .with_display_mode(PasswordDisplayMode::Masked)
                    .with_validator(min_length!(8))
                    .with_formatter(&|_| String::from("Input received"))
                    .with_help_message(
                        "Remeber this key, you will need it to decrypt your profile later",
                    )
                    .with_custom_confirmation_error_message("The keys don't match.")
                    .prompt();

                let user_key = if let Err(e) = prompt {
                    println!("{}: {}", "Error".red(), e);
                    return;
                } else {
                    prompt.unwrap()
                };

                create_profile(command_args.args[0].clone(), None, user_key)
            }
            Command::Add(command_args) => {
                if command_args.args.len() <= 1 {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                let mut profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                    p
                } else {
                    return;
                };

                for (count, arg) in command_args.args[1..].iter().enumerate() {
                    if count > command_args.args.len() - 2 {
                        break;
                    }

                    let mut split = arg.split('=');

                    let key = split.next();
                    let value = split.next();

                    if key.is_none() || value.is_none() {
                        println!("{}: Can not parse arguments", "Error".red());
                        println!(
                            "{}",
                            "Arguments should be in the format of key=value".bold()
                        );
                        return;
                    }

                    if profile.envs.contains_key(key.unwrap()) {
                        println!("{}: Key already exists in profile use the update command to update the value", "Error".red());
                        return;
                    }

                    profile.add_env(key.unwrap().to_owned(), value.unwrap().to_owned());
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes(get_userkey());
            }

            Command::Load(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();
                let profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                    p
                } else {
                    return;
                };

                profile.load_profile();
            }

            Command::Unload(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();
                let profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                    p
                } else {
                    return;
                };

                profile.unload_profile();
            }

            Command::Remove(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                if command_args.args.len() == 1 {
                    delete_profile(command_args.args[0].clone());
                } else {
                    let profile_name = command_args.args[0].clone();
                    let mut profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                        p
                    } else {
                        return;
                    };

                    for arg in command_args.args[1..].iter() {
                        profile.remove_env(arg.to_owned());
                    }

                    println!("{}", "Applying Changes".green());
                    profile.push_changes(get_userkey());
                }
            }
            Command::List(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                if profile_name == "profiles" {
                    list_profiles()
                } else {
                    let profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                        p
                    } else {
                        return;
                    };

                    profile.list_envs();
                }
            }

            Command::Update(command_args) => {
                if command_args.args.len() <= 1 {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                let mut profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                    p
                } else {
                    return;
                };

                for arg in command_args.args[1..].iter() {
                    let mut split = arg.split('=');

                    let key = split.next();
                    let value = split.next();

                    if key.is_none() || value.is_none() {
                        println!("{}: Can not parse arguments", "Error".red());
                        println!(
                            "{}",
                            "Arguments should be in the format of key=value".bold()
                        );
                        return;
                    }

                    if profile.envs.contains_key(key.unwrap()) {
                        profile.edit_env(key.unwrap().to_owned(), value.unwrap().to_owned())
                    } else {
                        println!(
                            "{}: Key does not exists in profile use the `add` command to add the key",
                            "Error".red()
                        );
                        return;
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes(get_userkey());
            }

            Command::Export(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();
                let mut file_name = String::from(".env");

                if command_args.args.len() > 1 {
                    file_name = command_args.args[1].clone();
                }

                let profile = if let Some(p) = get_profile(profile_name, get_userkey()) {
                    p
                } else {
                    return;
                };

                profile.export_envs(file_name);
            }

            Command::Import(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                if command_args.args.len() < 2 {
                    println!("{}: Please provide a profile name", "Error".red());
                    return;
                }

                let profile_name = command_args.args[1].clone();

                if check_profile(profile_name.clone()) {
                    println!("{}: Profile already exists", "Error".red());
                    return;
                }

                if Url::parse(command_args.args[0].as_str()).is_ok() {
                    download_profile(command_args.args[0].clone(), profile_name);
                    return;
                }

                let file_path = command_args.args[0].clone();
                let profile_name = command_args.args[1].clone();

                import_profile(file_path, profile_name);
            }

            Command::Version(command_args) => {
                if command_args.args.is_empty() {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                } else if command_args.args[0] == "verbose" {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                    println!("{} {}", "Build Timestamp".green(), env!("BUILD_TIMESTAMP"));
                    println!("{} {}", "Author".green(), env!("CARGO_PKG_AUTHORS"));
                    println!("{} {}", "License".green(), env!("CARGO_PKG_LICENSE"));
                    println!("{} {}", "Repository".green(), env!("CARGO_PKG_REPOSITORY"));
                } else {
                    println!("{}: Invalid argument", "Error".red());
                }
            }
        }
    }
}