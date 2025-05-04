use lsm_tree::skiplist_map::SkipListMap;

fn main() {
    // Use crossbeam_skiplist::SkipMap via our alias
    let skiplist = SkipListMap::new();
    let keys = ["apple", "banana", "cherry", "date", "fig"];
    // Insert keys into the skiplist
    for &key_str in &keys {
        let key = key_str.to_string();
        skiplist.insert(key.clone(), key.clone());
    }

    for &key_str in &keys {
        let key = key_str.to_string();
        match skiplist.get(&key) {
            Some(entry) => println!("Got {}", entry.value()),
            None => {
                eprintln!("Key {} not found", key_str);
                return;
            }
        }
    }

    println!("All tests passed");
}
