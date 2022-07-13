use std::{
    collections::HashMap,
    fs::File,
    io,
    io::{BufRead, BufReader},
    num,
};

extern crate quick_error;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum VectorLoadError {
        MissingHeader {
            display("expected a header line with «number of vectors» «dimensionality»")
        }
        WrongHeader {
            display("expected header line to be in the format «number of vectors» «dimensionality»")
        }
        ParseIntError(error: num::ParseIntError) {
            from(error: num::ParseIntError) -> (error)
            source(error)
            display("failed to parse header element: {}", error)
        }
        MissingWord {
            display("expected word at start of line")
        }
        ParseFloatError(error: num::ParseFloatError) {
            from(error: num::ParseFloatError) -> (error)
            source(error)
            display("failed to parse vector element: {}", error)
        }
        WrongDimensionality(expected: usize, actual: usize) {
            display("expected a vector of dimensionality {}, got {}", expected, actual)
        }
        MissingVectors(expected: usize, actual: usize) {
            display("expected to load {} vectors, got {}", expected, actual)
        }
        IOError(error: io::Error) {
            from(error: io::Error) -> (error)
            source(error)
            display("failed to open file: {}", error)
        }
    }
}

pub type Vector = Vec<f64>;
pub type Vectors = HashMap<String, Vector>;

fn parse_header(line: &str) -> Result<(usize, usize), VectorLoadError> {
    let mut elements = line.split(' ');

    let rows = elements
        .next()
        .ok_or(VectorLoadError::WrongHeader)?
        .parse()?;

    let dimensionality = elements
        .next()
        .ok_or(VectorLoadError::WrongHeader)?
        .parse()?;

    match elements.next() {
        // If there is still an element in the iterator, then the header is too long
        Some(_) => Err(VectorLoadError::WrongHeader),
        None => Ok((rows, dimensionality)),
    }
}

fn parse_vector(dimensionality: usize, line: &str) -> Result<(String, Vector), VectorLoadError> {
    let mut elements = line.split(' ');

    let word = elements.next().ok_or(VectorLoadError::MissingWord)?;

    let mut vector = Vec::with_capacity(dimensionality);
    for element in elements {
        vector.push(element.parse()?);
    }

    if vector.len() == dimensionality {
        Ok((word.to_string(), vector))
    } else {
        Err(VectorLoadError::WrongDimensionality(dimensionality, vector.len()))
    }
}

pub fn load_vectors() -> Result<Vectors, VectorLoadError> {
    println!("Loading vectors");

    let file = File::open(env!("WORD2VEC_DATA"))?;
    let mut lines = BufReader::new(file).lines();

    let header_line = lines.next().ok_or(VectorLoadError::MissingHeader)??;
    let (rows, dimensionality) = parse_header(&header_line)?;

    let mut vectors = HashMap::with_capacity(rows);
    for line in lines {
        let (word, vector) = parse_vector(dimensionality, &line?)?;
        vectors.insert(word, vector);
    }

    if vectors.len() == rows {
        Ok(vectors)
    } else {
        Err(VectorLoadError::MissingVectors(rows, vectors.len()))
    }
}

fn add_vectors(a: &mut Vector, b: &Vector) {
    for (a_value, b_value) in a.iter_mut().zip(b) {
        *a_value += b_value;
    }
}

fn sum_vectors(vectors: &[Vector]) -> Option<Vector> {
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
        *value /= scalar;
    }
}

fn mean_vectors(vectors: &[Vector]) -> Option<Vector> {
    let mut total = sum_vectors(vectors)?;
    let count = vectors.len() as f64;
    divide_vector(&mut total, count);
    Some(total)
}

pub fn utterance_to_vector(vectors: &Vectors, utterance: &str) -> Option<Vector> {
    let mut results: Vec<Vector> = Vec::new();

    // TODO: Tokenize properly
    for word in utterance.split(' ') {
        let word = word.to_lowercase();
        if let Some(vector) = vectors.get(&word) {
            results.push(vector.clone());
        }
    }

    mean_vectors(&results)
}
