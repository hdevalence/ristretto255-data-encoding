extern crate curve25519_dalek;
extern crate rand;

use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};

use rand::{thread_rng, RngCore};

/// Encodes up to 30 bytes as an element of the ristretto255 group.
///
/// It's not possible to straightforwardly use Elligator for this
/// purpose, because we want the encoding to be invertible: Elligator
/// maps field elements to points on the Jacobi quartic, but to get a
/// point on the ristretto255 group we then apply the quotient.  While the
/// Elligator map is invertible when considered as a map to the Jacobi
/// quartic, it is not invertible once the quotient is applied:
/// that two different internal representatives of the same point
/// may be the images of different field elements.
///
/// Instead, we encode 30 bytes at a time using increment-and-test.
///
/// This function computes a sequence of 32-byte candidate encodings, of
/// the form ` i || data || 0`, where i is a counter running over even
/// numbers 0,2,4,...,254. The first candidate which is a valid encoding
/// is the canonical representative of `data`.
///
/// Each candidate has a 1/4 chance of being a valid encoding, so the
/// probability of *not* finding a representative after k trials is
/// (3/4)**k.  The chance of not finding a representative after 128
/// trials is 2**(lg(3/4)*128) < 2**(-53).
///
/// The number of trials before success is a geometric distribution with
/// probability p = 1/4, so the expected number of trials is
/// (1-1/4)/(1/4) = 3.
///
/// In the extremely unlikely event that no candidate is found after 128
/// trials, we can begin incrementing the high byte from 0 (its initial
/// value) by 1s up to 64, giving 128*64 trials in total and cutting the
/// failure probability to 2**(lg(3/4)*128*64) ~= 2**(-3400) ~= 0.
fn encode_bytes_in_point(data: &[u8]) -> (RistrettoPoint, usize) {
    assert!(data.len() <= 30);
    let mut bytes = [0u8; 32];
    bytes[1..1 + data.len()].copy_from_slice(data);
    for j in 0..64 {
        bytes[31] = j as u8;
        for i in 0..128 {
            bytes[0] = 2 * i as u8;
            if let Some(point) = CompressedRistretto(bytes).decompress() {
                return (point, i + j * 128);
            }
        }
    }
    panic!("a very unlikely event occurred");
}

fn run_trial() -> usize {
    let mut bytes = [0u8; 30];
    thread_rng().fill_bytes(&mut bytes);

    encode_bytes_in_point(&bytes).1
}

fn collect_data(samples: usize) -> Vec<(usize, usize)> {
    let mut trial_counts = vec![0usize; 128];
    for _ in 0..samples {
        trial_counts[run_trial() as usize] += 1;
    }
    trial_counts.iter().cloned().enumerate().collect()
}

fn main() {
    let data = b"hello world";
    println!("{:?}", encode_bytes_in_point(data).0.compress());
    let data = b"hello world 2: hello worlder";
    println!("{:?}", encode_bytes_in_point(data).0.compress());
    let data = b"hello world 3: another world";
    println!("{:?}", encode_bytes_in_point(data).0.compress());
    let data = collect_data(10_000);
    println!("{:?}", data);
}
