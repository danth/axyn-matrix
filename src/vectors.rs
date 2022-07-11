use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::num;

#[derive(Debug)]
pub enum VectorLoadError {
    MissingHeaderError,
    HeaderLengthError,
    ParseIntError(num::ParseIntError),
    MissingWordError,
    ParseFloatError(num::ParseFloatError),
    WrongDimensionalityError { actual: usize, expected: usize },
    MissingVectorsError { actual: usize, expected: usize },
    IOError(io::Error)
}
impl From<num::ParseIntError> for VectorLoadError {
    fn from(error: num::ParseIntError) -> VectorLoadError {
        VectorLoadError::ParseIntError(error)
    }
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
            VectorLoadError::MissingHeaderError => write!(f, "expected a header line with «number of vectors» «dimensionality»"),
            VectorLoadError::HeaderLengthError => write!(f, "nexpected header line to be in the format «number of vectors» «dimensionality»"),
            VectorLoadError::ParseIntError(error) => write!(f, "failed to parse header element: {}", error),
            VectorLoadError::MissingWordError => write!(f, "expected word at start of line"),
            VectorLoadError::ParseFloatError(error) => write!(f, "failed to parse vector element: {}", error),
            VectorLoadError::WrongDimensionalityError{ actual, expected } =>
                write!(f, "expected a vector of dimensionality {}, got {}", expected, actual),
            VectorLoadError::MissingVectorsError{ actual, expected } =>
                write!(f, "expected to load {} vectors, found {}", expected, actual),
            VectorLoadError::IOError(error) => write!(f, "failed to open file: {}", error)
        }
    }
}

pub type Vector = Vec<f64>;
pub type Vectors = HashMap<String, Vector>;

fn parse_header(line: &str) -> Result<(usize, usize), VectorLoadError> {
    let mut elements = line.split(" ");

    let rows = elements
        .next().ok_or(VectorLoadError::HeaderLengthError)?
        .parse()?;

    let dimensionality = elements
        .next().ok_or(VectorLoadError::HeaderLengthError)?
        .parse()?;

    match elements.next() {
        // If there is still an element in the iterator, then the header is too long
        Some(_) => Err(VectorLoadError::HeaderLengthError),
        None => Ok((rows, dimensionality))
    }
}

fn parse_vector(
    dimensionality: usize,
    line: &str
) -> Result<(String, Vector), VectorLoadError> {
    let mut elements = line.split(" ");

    let word = elements.next().ok_or(VectorLoadError::MissingWordError)?;

    let mut vector = Vec::with_capacity(dimensionality);
    for element in elements {
        vector.push(element.parse()?);
    }

    if vector.len() == dimensionality {
        Ok((word.to_string(), vector))
    } else {
        Err(VectorLoadError::WrongDimensionalityError {
            actual: vector.len(),
            expected: dimensionality
        })
    }
}

pub fn load_vectors() -> Result<Vectors, VectorLoadError> {
    println!("Loading vectors");

    let file = File::open(env!("WORD2VEC_DATA"))?;
    let mut lines = BufReader::new(file).lines();

    let header_line = lines.next().ok_or(VectorLoadError::MissingHeaderError)??;
    let (rows, dimensionality) = parse_header(&header_line)?;

    let mut vectors = HashMap::with_capacity(rows);
    for line in lines {
        let (word, vector) = parse_vector(dimensionality, &line?)?;
        vectors.insert(word, vector);
    }

    if vectors.len() == rows {
        Ok(vectors)
    } else {
        Err(VectorLoadError::MissingVectorsError {
            actual: vectors.len(),
            expected: rows
        })
    }
}

fn add_vectors(a: &mut Vector, b: &Vector) {
    for (a_value, b_value) in a.iter_mut().zip(b) {
        *a_value = *a_value + b_value;
    }
}

fn sum_vectors(vectors: &Vec<Vector>) -> Option<Vector> {
    let mut iterator = vectors.iter();

    let head = iterator.next()?;
    let mut result = head.clone();

    for tail in iterator {
        add_vectors(&mut result, tail);
    }

    Some(result)
}

fn divide_vector(vector: &mut Vector, scalar: f64) {
    for value in vector.iter_mut() {
        *value = *value / scalar;
    }
}

fn mean_vectors(vectors: &Vec<Vector>) -> Option<Vector> {
    let mut total = sum_vectors(vectors)?;
    let count = vectors.len() as f64;
    divide_vector(&mut total, count);
    Some(total)
}

pub fn utterance_to_vector(vectors: &Vectors, utterance: &str) -> Option<Vector> {
    let mut results: Vec<Vector> = Vec::new();

    // TODO: Tokenize properly
    for word in utterance.split(" ") {
        let word = word.to_lowercase();
        if let Some(vector) = vectors.get(&word) {
            results.push(vector.clone());
        }
    }

    mean_vectors(&results)
}
