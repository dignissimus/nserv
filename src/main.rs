mod protocol;
mod server;

use server::Server;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use config::types::*;
use config::types::ScalarValue::*;
use config::types::Value::*;
use protocol::Location;
use std::thread::JoinHandle;

extern crate config;

fn main() {
    println!("Reservd!");
    let servers = get_servers();
    let mut server_threads: Vec<JoinHandle<()>> = Vec::new();
    // create a ServerManager and add the servers
    for server in servers {
        //temp
        server_threads.push(server.start());
    }
    for server in server_threads{
        server.join();
    }
}

fn get_servers() -> Vec<Server> {
    if let Ok(config_file) = File::open("server.config") {
        let mut reader = BufReader::new(config_file);
        parse_config(&mut reader)
    } else {
        eprintln!("Unable to read config file");
        Vec::new()
    }
}

fn parse_settings(settings: &SettingsList) -> Server {
    let port = get_int(&settings.get("port").expect("Please add the `port` field").value) as u16;
    let host = get_string(&settings.get("host").expect("Please add the `host` field").value);
    let location = if let Some(ref location_string) = settings.get("root") {
        Location::Document(get_string(&location_string.value))
    } else if let Some(ref address) = (*settings).get("proxy_address") {
        Location::RProxy(get_string(&address.value))
    } else {
        panic!("Please add a location")
    };
    println!("port: {}", port);
    Server { port: port, host: host, location: location }
}

fn parse_config(reader: &mut BufReader<File>) -> Vec<Server> {
    let mut servers: Vec<Server> = Vec::new();
    if let Ok(config) = config::reader::from_stream(reader) {
        if let Some(value) = config.lookup("servers") {
            // add get_list function?
            let config_servers = get_list(value);
            for server in config_servers {
                let settings = get_group(&server);
                servers.push(parse_settings(&settings));
            }
        }
    } else {
        eprintln!("Unable to parse config");
    }
    servers
}

fn get_scalar(value: &Value) -> &ScalarValue {
    if let Svalue(ref scalar) = *value {
        scalar
    } else {
        panic!("Expected a Scalar but found other type. Panicking")
    }
}

fn get_int(value: &Value) -> i32 {
    let scalar = get_scalar(value);
    if let Integer32(ref int) = *scalar {
        *int
    } else {
        0
    }
}

fn get_string(value: &Value) -> String {
    let scalar = get_scalar(value);
    if let Str(ref string) = *scalar {
        string.clone()
    } else {
        panic!("Expected a string but found other type. Panicking!")
    }
}

fn get_list(value: &Value) -> &ListValue {
    if let List(ref list) = *value {
        list
    } else {
        panic!("Expected a list but found other type. Panicking!")
    }
}

fn get_group(value: &Value) -> &SettingsList {
    if let Group(ref settings) = *value {
        settings
    } else {
        panic!("Expected a Group but found other type. Panicking!")
    }
}