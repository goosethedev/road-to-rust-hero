use actionkv::ActionKV;

#[cfg(target_os = "windows")]
const USAGE: &str = "
Usage:
    akv_mem.exe FILE list-keys
    akv_mem.exe FILE get KEY
    akv_mem.exe FILE delete KEY
    akv_mem.exe FILE insert KEY VALUE
    akv_mem.exe FILE update KEY VALUE
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
Usage:
    akv_mem FILE list-keys
    akv_mem FILE get KEY
    akv_mem FILE delete KEY
    akv_mem FILE insert KEY VALUE
    akv_mem FILE update KEY VALUE
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_name = args.get(1).expect(USAGE);
    let action = args.get(2).expect(USAGE);

    let path = std::path::Path::new(file_name);
    let mut store = ActionKV::open(path).expect("Unable to open file");
    store.load().expect("Unable to load data");

    match action.as_str() {
        "list-keys" => {
            for key in store.list_keys() {
                println!("Key: {}", key);
            }
        }
        "get" => {
            let key = args.get(3).expect(USAGE);
            let value = store.get(key.as_bytes()).unwrap().expect("Key not found");
            let value = String::from_utf8_lossy(&value);
            println!("Key: {} - Value: {}", key, value);
        }
        "insert" => {
            let key = args.get(3).expect(USAGE);
            let value = args.get(4).expect("Value not provided");
            store.insert(key.as_bytes(), value.as_bytes()).unwrap();
            println!("Key: {} - Value: {} | Successfully inserted.", key, value);
        }
        "update" => {
            let key = args.get(3).expect(USAGE);
            let value = args.get(4).expect("Value not provided");
            store.update(key.as_bytes(), value.as_bytes()).unwrap();
            println!("Key: {} - Value: {} | Successfully updated.", key, value);
        }
        "delete" => {
            let key = args.get(3).expect(USAGE);
            store.delete(key.as_bytes()).unwrap();
            println!("Key: {} | Successfully deleted.", key);
        }
        _ => eprint!("{}", &USAGE),
    };
}
