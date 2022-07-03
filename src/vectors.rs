use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::num;

#[derive(Debug)]
pub enum VectorLoadError {
    MissingWordError,
    ParseFloatError(num::ParseFloatError),
    IOError(io::Error)
}
impl From<num::ParseFloatError> for VectorLoadError {
    fn from(error: num::ParseFloatError) -> VectorLoadError {
        VectorLoadError::ParseFloatError(error)
    }
}
impl From<io::Error> for VectorLoadError {
    fn from(error: io::Error) -> VectorLoadError {
        VectorLoadError::IOError(error)
    }
}
impl fmt::Display for VectorLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VectorLoadError::MissingWordError => write!(f, "expected word at start of line"),
            VectorLoadError::ParseFloatError(error) => write!(f, "failed to parse vector element: {}", error),
            VectorLoadError::IOError(error) => write!(f, "failed to open file: {}", error)
        }
    }
}

pub type Vector = Vec<f64>;
pub type Vectors = HashMap<String, Vector>;

pub fn load_vectors() -> Result<Vectors, VectorLoadError> {
    let mut vectors = HashMap::new();

    let file = File::open(env!("WORD2VEC_DATA"))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let mut elements = line.split(" ");

        let word = elements.next()
            .ok_or(VectorLoadError::MissingWordError)?;

        let vector: Vector = elements.map(|n| n.parse())
            .collect::<Result<_, num::ParseFloatError>>()?;

        vectors.insert(word.to_string(), vector);
    }

    // TODO: Ensure that all vectors are the same size

    Ok(vectors)
}
