use instant_distance::HnswMap as HNSW;
use instant_distance::{Builder, Search};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Data type for the key-value store value's metadata.
pub type Data = HashMap<String, String>;

// This is the data structure that will be stored in
// the key-value store as the value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub embedding: Vec<f32>,
    pub data: Data,
}

// Arc and Mutex to share the key-value store
// across threads while ensuring exclusive access.
type KeyValue = Arc<Mutex<HashMap<String, Value>>>;

type Index = Arc<Mutex<Vec<HNSW<Value, String>>>>;

// db
pub struct Config {
    pub dimension: usize,
    pub token: String,
}

pub struct Server {
    pub config: Config,
    kvs: KeyValue,
    index: Index,
}

impl Server {
    pub fn new(config: Config) -> Server {
        let kvs: KeyValue = Arc::new(Mutex::new(HashMap::new()));
        let index: Index = Arc::new(Mutex::new(Vec::with_capacity(1)));

        Server { kvs, index, config }
    }

    // Native functionality handler.
    // These are the functions that handle the native
    // functionality of the database.
    // Example: get, set, delete, etc.

    pub fn get(&self, key: String) -> Result<Value, &str> {
        let kvs = self.kvs.lock().unwrap();
        kvs.get(&key).cloned().ok_or("The value is not found.")
    }

    pub fn set(&self, key: String, value: Value) -> Result<Value, &str> {
        if value.embedding.len() != self.config.dimension {
            return Err("The embedding dimension is invalid.");
        }

        let mut kvs = self.kvs.lock().unwrap();
        kvs.insert(key, value.clone());
        Ok(value)
    }

    pub fn delete(&self, key: String) -> Result<Value, &str> {
        let mut kvs = self.kvs.lock().unwrap();
        kvs.remove(&key).ok_or("The key doesn't exist.")
    }

    // Index functionality handler.
    pub fn build(
        &self,
        ef_search: usize,
        ef_construction: usize,
    ) -> Result<&str, &str> {
        // Clear the current index
        let mut index = self.index.lock().unwrap();
        index.clear();

        // Get the key-value store.
        let kvs = self.kvs.lock().unwrap();

        // Separate key-value to keys and values.
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in kvs.iter() {
            keys.push(key.clone());
            values.push(value.clone());
        }

        // Build and set the index.
        let _index = Builder::default()
            .ef_search(ef_search)
            .ef_construction(ef_construction)
            .build(values, keys);

        index.push(_index);
        Ok("The index is built successfully.")
    }

    pub fn search(
        &self,
        embedding: Vec<f32>,
        count: usize,
    ) -> Result<Vec<Data>, &str> {
        // Validate the dimension of the embedding.
        if embedding.len() != self.config.dimension {
            return Err("The embedding dimension is invalid.");
        }

        let _index = self.index.lock().unwrap();
        let index = match _index.first() {
            Some(index) => index,
            None => return Err("The index is not built yet."),
        };

        // Create a decoy value with the provided embedding.
        // Data is not needed for the search.
        let point = Value { embedding, data: HashMap::new() };

        let mut search = Search::default();
        let results = index.search(&point, &mut search);

        let mut data: Vec<Data> = Vec::new();
        for result in results {
            let value = result.point;
            data.push(value.data.clone());
        }

        data.truncate(count);

        Ok(data)
    }
}

// This is the implementation of the Point trait.
// This is needed by the library to calculate the distance
// between two vectors.
impl instant_distance::Point for Value {
    fn distance(&self, other: &Self) -> f32 {
        let mut sum = 0.0;

        // Implement Euclidean distance formula.
        for i in 0..self.embedding.len() {
            sum += (self.embedding[i] - other.embedding[i]).powi(2);
        }

        sum.sqrt()
    }
}
