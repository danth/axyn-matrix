use std::fmt;

extern crate hnsw;
use hnsw::{Hnsw, Searcher};
extern crate rand_pcg;
use rand_pcg::Pcg64;
extern crate space;
use space::{Metric, Neighbor};

extern crate sled;
use sled::Db;
extern crate serde_cbor;

use crate::vectors::Vector;

#[derive(Debug)]
pub enum Error {
    DatabaseError(sled::Error),
    SerdeError(serde_cbor::Error),
    MissingResponses
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
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DatabaseError(e) => write!(f, "Database error: {}", e),
            Error::SerdeError(e) => write!(f, "Serialization/deserialization error: {}", e),
            Error::MissingResponses => write!(f, "Responses should exist in the database, but they do not")
        }
    }
}

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

pub struct ResponseStore {
    database: Db,
    hnsw: Hnsw<Euclidean, Vector, Pcg64, 12, 24>,
    searcher: Searcher<u64>
}
impl ResponseStore {
    pub fn load() -> Result<Self, Error> {
        let database = sled::open("kv_store")?;

        let mut hnsw = Hnsw::new(Euclidean);
        let mut searcher = Searcher::default();

        for pair in database.iter() {
            let (serialized_vector, _) = pair?;
            let vector = serde_cbor::from_slice(&serialized_vector)?;
            hnsw.insert(vector, &mut searcher);
        }

        Ok(ResponseStore { database, hnsw, searcher })
    }

    pub fn insert(&mut self, vector: &Vector, response: &str) -> Result<(), Error> {
        let serialized_vector = serde_cbor::to_vec(vector)?;

        let mut responses = match self.database.get(&serialized_vector)? {
            Some(r) => (serde_cbor::from_slice::<Vec<String>>(&r)?).clone(),
            None => Vec::new()
        };

        responses.push(response.to_string());

        let serialized_responses = serde_cbor::to_vec(&responses)?;
        self.database.insert(serialized_vector, serialized_responses)?;

        self.hnsw.insert(vector.clone(), &mut self.searcher);

        Ok(())
    }

    pub fn respond(&mut self, vector: &Vector) -> Result<String, Error> {
        let mut neighbours = [Neighbor {
            index: !0,
            distance: !0
        }];

        self.hnsw.nearest(&vector, 24, &mut self.searcher, &mut neighbours);

        let vector = self.hnsw.feature(neighbours[0].index);
        let serialized_vector = serde_cbor::to_vec(&vector)?;

        let responses = self.database.get(&serialized_vector)?;
        let responses = responses.ok_or(Error::MissingResponses)?;
        let responses: Vec<String> = serde_cbor::from_slice(&responses)?;

        Ok(responses[0].clone())
    }
}
