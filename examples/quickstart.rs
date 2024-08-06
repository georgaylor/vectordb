use sahomedb::prelude::*;

fn main() {
    // Vector dimension must be uniform.
    let dimension = 128;

    // Replace with your own data.
    let records = Record::many_random(dimension, 100);

    // Create a vector collection.
    let config = Config::default();
    let collection = Collection::build(&config, &records).unwrap();

    // Optionally save the collection to persist it.
    let mut db = Database::new("data/test").unwrap();
    db.save_collection("vectors", &collection).unwrap();

    // Search for the nearest neighbors.
    let query = Vector::random(dimension);
    let result = collection.search(&query, 5).unwrap();
    println!("Nearest ID: {}", result[0].id);
}
