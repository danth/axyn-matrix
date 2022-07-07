use std::fmt;
use std::sync::{Arc, RwLock};

extern crate dirs;

extern crate hnsw;
use hnsw::{Hnsw, Searcher};
extern crate space;
use space::{Metric, Neighbor};

extern crate sled;
use sled::Db;
extern crate serde_cbor;

extern crate rand;
use rand::seq::SliceRandom;
use rand::rngs::StdRng;

use crate::matrix::Body;
use crate::vectors::{Vector, Vectors, load_vectors, VectorLoadError, utterance_to_vector};

#[derive(Debug)]
pub enum Error {
    DatabaseError(sled::Error),
    SerdeError(serde_cbor::Error),
    VectorLoadError(VectorLoadError),
    NoResponses,
    MissingResponses,
    NoPromptVector
}
impl From<sled::Error> for Error {
    fn from(error: sled::Error) -> Error {
        Error::DatabaseError(error)
    }
}
impl From<serde_cbor::Error> for Error {
    fn from(error: serde_cbor::Error) -> Error {
        Error::SerdeError(error)
    }
}
impl From<VectorLoadError> for Error {
    fn from(error: VectorLoadError) -> Error {
        Error::VectorLoadError(error)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DatabaseError(e) => write!(f, "Database error: {}", e),
            Error::SerdeError(e) => write!(f, "Serialization/deserialization error: {}", e),
            Error::VectorLoadError(e) => write!(f, "Error loading vectors: {}", e),
            Error::NoResponses => write!(f, "No responses exist in the database"),
            Error::MissingResponses => write!(f, "Responses should exist in the database, but they do not"),
            Error::NoPromptVector => write!(f, "The prompt did not contain any words with known vectors")
        }
    }
}

#[derive(Clone, Debug)]
struct Euclidean;
impl Metric<Vector> for Euclidean {
    type Unit = u64;
    fn distance(&self, a: &Vector, b: &Vector) -> u64 {
        a.iter().zip(b.iter())
            .map(|(&a, &b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
            .to_bits()
    }
}

#[derive(Clone)]
pub struct ResponseStore {
    vectors: Arc<Vectors>,
    database: Db,
    hnsw_lock: Arc<RwLock<Hnsw<Euclidean, Vector, StdRng, 12, 24>>>,
    searcher_lock: Arc<RwLock<Searcher<u64>>>
}
impl ResponseStore {
    pub fn load() -> Result<Self, Error> {
        let vectors = load_vectors()?;
        let vectors_arc = Arc::new(vectors);

        println!("Opening database");
        let mut path = dirs::home_dir().expect("Finding home directory");
        path.push("responses");
        let database = sled::open(path)?;

        println!("Preparing HNSW");
        let mut hnsw = Hnsw::new(Euclidean);
        let mut searcher = Searcher::default();

        for pair in database.iter() {
            let (serialized_vector, _) = pair?;
            let vector = serde_cbor::from_slice(&serialized_vector)?;
            hnsw.insert(vector, &mut searcher);
        }

        let hnsw_lock = Arc::new(RwLock::new(hnsw));
        let searcher_lock = Arc::new(RwLock::new(searcher));

        Ok(ResponseStore { vectors: vectors_arc, database, hnsw_lock, searcher_lock })
    }

    pub fn insert(&self, prompt: &str, response: Body) -> Result<(), Error> {
        let vector = utterance_to_vector(&self.vectors, prompt)
            .ok_or(Error::NoPromptVector)?;
        let serialized_vector = serde_cbor::to_vec(&vector)?;

        let mut responses = match self.database.get(&serialized_vector)? {
            Some(r) => (serde_cbor::from_slice::<Vec<Body>>(&r)?).clone(),
            None => Vec::new()
        };

        responses.push(response);

        let serialized_responses = serde_cbor::to_vec(&responses)?;
        self.database.insert(serialized_vector, serialized_responses)?;

        let mut hnsw = self.hnsw_lock.write().unwrap();
        let mut searcher = self.searcher_lock.write().unwrap();
        hnsw.insert(vector.clone(), &mut searcher);

        Ok(())
    }

    pub fn respond(&self, prompt: &str) -> Result<Body, Error> {
        let vector = utterance_to_vector(&self.vectors, prompt)
            .ok_or(Error::NoPromptVector)?;

        let mut neighbours = [Neighbor {
            index: !0,
            distance: !0
        }];

        let hnsw = self.hnsw_lock.read().unwrap();
        let mut searcher = self.searcher_lock.write().unwrap();
        hnsw.nearest(&vector, 24, &mut searcher, &mut neighbours);

        if neighbours[0].index == !0 {
            return Err(Error::NoResponses);
        }

        let vector = hnsw.feature(neighbours[0].index);
        let serialized_vector = serde_cbor::to_vec(&vector)?;

        let responses = self.database.get(&serialized_vector)?;
        let responses = responses.ok_or(Error::MissingResponses)?;
        let responses: Vec<Body> = serde_cbor::from_slice(&responses)?;

        let response = responses.choose(&mut rand::thread_rng());
        let response = response.ok_or(Error::MissingResponses)?;

        Ok(response.clone())
    }
}
