extern crate hashmap;
use hashmap::HashMap;

fn main() {
    let map: HashMap<&str, i32> = [("Norway", 100), ("Denmark", 50), ("Iceland", 10)]
        .iter()
        .cloned()
        .collect();

    // use the values sorted in map
    println!("{:?}", map)
}
