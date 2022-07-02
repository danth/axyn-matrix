mod vectors;
use vectors::load_vectors;

fn main() {
    println!("Loading vectors");
    let vectors = load_vectors().expect("Error loading vectors");

    let vector = vectors.get("fish").expect("Getting vector for demo");
    println!("fish: {:?}", vector);
}
