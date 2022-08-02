use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::Command;

const CONFIG_FILE_PATH: &str = "./src/config.yaml";

pub fn read_config() -> Result<HashMap<String, HashMap<String, String>>, std::io::Error> {
    let f = fs::read_to_string(CONFIG_FILE_PATH)?;
    let config: Result<HashMap<String, HashMap<String, String>>, serde_yaml::Error> =
        serde_yaml::from_str(&f);
    let config = match config {
        Ok(v) => v,
        Err(e) => return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse config file: {}", e),
        )),
    };

    return Ok(config);
}

pub fn run_command(config: &HashMap<String, HashMap<String, String>>, program: &String, button: &String) -> Result<(), std::io::Error>{
    let bindings = match config.get(program) {
        Some(_) => &config[program],
        None => &config["_default"],
    };
    let command = match bindings.get(button) {
        Some(_) => &bindings[button],
        None => match &config["_default"].get(button) {
            Some(_) => &config["_default"][button],
            None => "",
        },
    };

    let output = Command::new("bash").arg("-c").arg(command).output()?;
    if !output.stderr.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Failed to run command: `{}`, error: {}",
                command,
                String::from_utf8(output.stderr).unwrap_or("error: invalid utf8 sequence".to_owned()),
            ),
        ));
    }

    println!("{}: {:?}", program, command);
    return Ok(());
}

pub fn stop_command(config: &HashMap<String, HashMap<String, String>>, button: &String) -> Result<(), std::io::Error>{

    return Ok(());
}