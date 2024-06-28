use std::{rc::Rc, sync::mpsc::Sender};

use num_bigint::{BigInt, RandBigInt, ToBigInt};
use num_traits::{One, Zero};
use rand::rngs::ThreadRng;

const NUM_THREADS: usize = 8;

struct PrimeMessage(i8, BigInt);

fn miller_test(d: &BigInt, n: &BigInt, rng: &mut ThreadRng) -> bool {
    let one: BigInt = One::one();
    let two: BigInt = &one + &one;

    let a = rng.gen_bigint_range(&two, &(n - &two)) + &one;
    let mut x = BigInt::modpow(&a, d, n);

    if x == one || x == n - &one {
        return true;
    }

    let mut d = d.clone();
    while d != n - &one {
        x = (&x * &x) % n;
        d *= &two;

        if x == one {
            return false;
        }
        if x == n - &one {
            return true;
        }
    }

    false
}

fn is_prime(num: &BigInt) -> bool {
    let one: BigInt = One::one();
    if num <= &one {
        return false;
    }

    let mut d = num - &one;
    while &d % 2 == Zero::zero() {
        d /= BigInt::from(2);
    }

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        if !miller_test(&d, num, &mut rng) {
            return false;
        }
    }
    true
}

fn test_primes(start: &BigInt, step: i32, id: i8, tx: Sender<PrimeMessage>) {
    let mut testing = start.clone();
    loop {
        let res = is_prime(&testing);
        let idd = if res { id } else { -id };
        if tx.send(PrimeMessage(idd, testing.clone())).is_err() {
            break;
        }
        testing += step * 2;
    }
}

fn main() {
    let mut start = BigInt::from(3);
    let start_time = std::time::Instant::now();
    let mut found = vec![];
    found.push(BigInt::from(2));
    let (tx, rx) = std::sync::mpsc::channel();

    for id in 1..NUM_THREADS + 1 {
        let tx = tx.clone();
        let my_start = start.clone();
        std::thread::spawn(move || {
            test_primes(&my_start, NUM_THREADS as i32, id as i8, tx);
        });
        start += 2;
    }

    let mut done = [false; NUM_THREADS];

    let max = 10_i32.pow(7).to_bigint().unwrap();
    for prime in rx {
        if prime.1 > max {
            let idx = prime.0.unsigned_abs() as usize - 1;
            done[idx] = true;
            if done.iter().all(|&x| x) {
                break;
            }
        }
        if prime.0 > 0 {
            found.push(prime.1);
        }
    }

    println!(
        "Found {} prime numbers in {:?}",
        found.len(),
        start_time.elapsed()
    );
}
