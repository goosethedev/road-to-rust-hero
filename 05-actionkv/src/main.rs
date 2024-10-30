use std::{cell::RefCell, rc::Rc};

use actionkv::ActionKV;
use easy_repl::{command, CommandStatus, Repl};

#[cfg(target_os = "windows")]
const USAGE: &str = "
An file-based, log-structured, append-only key-value database.

Usage: akv.exe FILE
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
An file-based, log-structured, append-only key-value database.

Usage: akv FILE
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_name = args.get(1).expect(USAGE);
    let path = std::path::Path::new(file_name);
    let store = ActionKV::new(path).expect("Unable to open file");

    println!("{}[2J", 27 as char);
    repl(store);
}

fn repl(store: ActionKV) {
    let store = Rc::new(RefCell::new(store));

    // TODO: Bruh
    let store_1 = Rc::clone(&store);
    let store_2 = Rc::clone(&store);
    let store_3 = Rc::clone(&store);
    let store_4 = Rc::clone(&store);
    let store_5 = Rc::clone(&store);

    let mut repl = Repl::builder()
        .add(
            "list-keys",
            command! { "List all the keys available",
            () => || {
                for key in store_1.borrow().list_keys() {
                    println!("{}", key);
                }
                Ok(CommandStatus::Done)
            }},
        )
        .add(
            "get",
            command! { "Get the value of a key",
            (key: String) => |key: String| {
                if let Some(value) = store_2.borrow().get(key.as_bytes()).unwrap() {
                    let value = String::from_utf8_lossy(&value);
                    println!("{} -> {}", key, value);
                } else {
                    println!("Error: Key not found");
                }
                Ok(CommandStatus::Done)
            }},
        )
        .add(
            "insert",
            command! { "Insert a key-value pair",
            (key: String, value: String) => |key: String, value: String| {
                store_3.borrow_mut().insert(key.as_bytes(), value.as_bytes()).unwrap();
                println!("{} -> {} | Successfully inserted", key, value);
                Ok(CommandStatus::Done)
            }},
        )
        .add(
            "update",
            command! { "Update a key-value pair",
            (key: String, value: String) => |key: String, value: String| {
                if let Some(()) = store_4.borrow_mut().update(key.as_bytes(), value.as_bytes()).unwrap() {
                    println!("{} -> {} | Successfully updated", key, value);
                } else {
                    println!("Error: Key not found");
                };
                Ok(CommandStatus::Done)
            }},
        )
        .add(
            "delete",
            command! { "Delete a key-value pair",
            (key: String) => |key: String| {
                if let Some(()) = store_5.borrow_mut().delete(key.as_bytes()).unwrap() {
                    println!("{} | Successfully deleted", key);
                } else {
                    println!("Error: Key not found");
                };
                Ok(CommandStatus::Done)
            }},
        )
        .build()
        .expect("Failed to create REPL");

    repl.run().expect("Failed to run REPL");
}
