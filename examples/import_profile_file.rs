// In this example we import the profile passed as an argument to the program and then save it

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: <file_path> <profile_name>");
        return;
    }

    let file_path = args[1].to_string(); // The first argument is the url of the profile filehe second argument is the key used to encrypt the profile file
    let profile_name = args[2].to_string(); // The third argument is the name that the profile will be saved as

    envio::import_profile(file_path, profile_name.clone());

    // Check that the profile was downloaded correctly
    // Make sure you have the key that was used to encrypt the profile file or else you won't be able to decrypt it
    let key = String::from("asdasdasas");

    for (env_var, value) in &envio::get_profile(profile_name, key).unwrap().envs {
        println!("{}: {}", env_var, value);
    }
}