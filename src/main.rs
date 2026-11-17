use rand::prelude::*;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    // Optional first argument: number of binary digits, e.g. `cargo run --release -- 100000`.
    // Without it, a random length between 1 and 10 is used.
    let len: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| rand::thread_rng().gen_range(1..=10));

    // Generate random binary numbers
    let binary_number1 = remove_leading_zeros(generate_random_binary(len));
    let binary_number2 = remove_leading_zeros(generate_random_binary(len));

    if len <= 64 {
        println!("Binary1: {}", binary_number1);
        println!("Binary2: {}", binary_number2);
    } else {
        println!("Binary1 and Binary2: {} digits each (not printed)", len);
    }

    // Sequential sum (the reference answer)
    let start = Instant::now();
    let seq_result = add_binary(&binary_number1, &binary_number2);
    let seq_time = start.elapsed();

    // First parallel algorithm: only works below 64 digits (u64 overflow)
    if binary_number1.len() < 64 && binary_number2.len() < 64 {
        let start = Instant::now();
        let parallel_result = par_add_binary1(&binary_number1, &binary_number2);
        println!("\n0. Parallel sum 1 took {:?}", start.elapsed());
        assert_eq!(parallel_result, seq_result, "par_add_binary1 gives a wrong sum");
    } else {
        println!("\n0. Parallel sum 1 skipped (needs fewer than 64 digits)");
    }

    // Second parallel algorithm
    let start = Instant::now();
    let parallel_result = par_add_binary(&binary_number1, &binary_number2);
    println!("1. Parallel sum 2 took {:?}", start.elapsed());
    assert_eq!(parallel_result, seq_result, "par_add_binary gives a wrong sum");

    println!("2. Sequential sum took {:?}", seq_time);
    if len <= 64 {
        println!("\nSum: {}", seq_result);
    }
    println!("All algorithms give the same sum.");
}

/// Sequential binary addition.
/// Works on bytes: `as_bytes()[i]` is O(1), while `chars().nth(i)` walks the string and is O(i).
fn add_binary(a: &str, b: &str) -> String {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let max_len = a.len().max(b.len());
    let mut result = Vec::with_capacity(max_len + 1); // digits, least significant first
    let mut carry = 0u8;
    for i in 0..max_len {
        let digit_a = if i < a.len() { a[a.len() - 1 - i] - b'0' } else { 0 };
        let digit_b = if i < b.len() { b[b.len() - 1 - i] - b'0' } else { 0 };
        let sum = digit_a + digit_b + carry;
        result.push(b'0' + sum % 2);
        carry = sum / 2;
    }
    if carry == 1 {
        result.push(b'1');
    }
    result.reverse();
    remove_leading_zeros(String::from_utf8(result).unwrap())
}

fn generate_random_binary(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| rng.gen_range(0..2).to_string())
        .collect()
}

fn remove_leading_zeros(binary_str: String) -> String {
    let trimmed_str: String = binary_str.trim_start_matches('0').to_string();
    if trimmed_str.is_empty() {
        "0".to_string()
    } else {
        trimmed_str
    }
}

// First algo of parallel sum binary
// Limit: a u64 holds 64 bits, so each number needs fewer than 64 digits (the sum must fit too).
fn par_add_binary1(a: &str, b: &str) -> String {
    assert!(
        a.len() < 64 && b.len() < 64,
        "par_add_binary1 only works with fewer than 64 binary digits"
    );
    // Convert binary strings to decimals in parallel
    let sum_decimal: u64 = [a, b]
        .par_iter()
        .map(|binary| binary_to_decimal(binary))
        .sum();

    // Convert the sum decimal to binary
    decimal_to_binary(sum_decimal)
}

fn binary_to_decimal(binary: &str) -> u64 {
    u64::from_str_radix(binary, 2).expect("Invalid binary number")
}

fn decimal_to_binary(decimal: u64) -> String {
    format!("{:b}", decimal)
}

// Second algo of parallel sum binary (carry-select)
//
// The carry is the hard part: chunk k needs the carry coming out of chunk k-1.
// Trick: add every chunk TWICE in parallel, once assuming carry-in = 0 and once
// carry-in = 1. Then chain the carries chunk by chunk (only as many steps as there
// are chunks) and keep the matching result of each chunk.
fn par_add_binary(a: &str, b: &str) -> String {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let max_len = a.len().max(b.len());
    let threads = rayon::current_num_threads();
    let chunk_size = ((max_len + threads - 1) / threads).max(1);
    let digit = |s: &[u8], i: usize| if i < s.len() { s[s.len() - 1 - i] - b'0' } else { 0 };

    let starts: Vec<usize> = (0..max_len).step_by(chunk_size).collect();

    // For each chunk: (digits for carry-in 0, carry-out, digits for carry-in 1, carry-out),
    // digits least significant first.
    let chunks: Vec<(Vec<u8>, u8, Vec<u8>, u8)> = starts
        .par_iter()
        .map(|&start| {
            let end = (start + chunk_size).min(max_len);
            let add_chunk = |mut carry: u8| {
                let mut digits = Vec::with_capacity(end - start);
                for i in start..end {
                    let sum = digit(a, i) + digit(b, i) + carry;
                    digits.push(b'0' + sum % 2);
                    carry = sum / 2;
                }
                (digits, carry)
            };
            let (digits0, carry0) = add_chunk(0);
            let (digits1, carry1) = add_chunk(1);
            (digits0, carry0, digits1, carry1)
        })
        .collect();

    // Chain the carries (a handful of steps, not one per digit)
    let mut result = Vec::with_capacity(max_len + 1);
    let mut carry = 0u8;
    for (digits0, carry0, digits1, carry1) in chunks {
        if carry == 0 {
            result.extend(digits0);
            carry = carry0;
        } else {
            result.extend(digits1);
            carry = carry1;
        }
    }
    if carry == 1 {
        result.push(b'1');
    }
    result.reverse();
    remove_leading_zeros(String::from_utf8(result).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readme_example() {
        assert_eq!(add_binary("111", "110"), "1101");
        assert_eq!(par_add_binary("111", "110"), "1101");
        assert_eq!(par_add_binary1("111", "110"), "1101");
    }

    #[test]
    fn edge_cases() {
        for (a, b, want) in [("0", "0", "0"), ("1", "0", "1"), ("1", "1", "10"), ("11", "01", "100"), ("1111", "1", "10000")] {
            assert_eq!(add_binary(a, b), want, "{a}+{b}");
            assert_eq!(par_add_binary(a, b), want, "{a}+{b}");
        }
    }

    #[test]
    fn carry_across_chunks() {
        // 1111...1 + 1 carries through every chunk
        let a = "1".repeat(1000);
        let want = format!("1{}", "0".repeat(1000));
        assert_eq!(add_binary(&a, "1"), want);
        assert_eq!(par_add_binary(&a, "1"), want);
    }

    #[test]
    fn random_inputs_match_u128() {
        let mut rng = rand::thread_rng();
        for _ in 0..5000 {
            let (la, lb) = (rng.gen_range(1..=100), rng.gen_range(1..=100));
            let a = remove_leading_zeros(generate_random_binary(la));
            let b = remove_leading_zeros(generate_random_binary(lb));
            let want = format!("{:b}", u128::from_str_radix(&a, 2).unwrap() + u128::from_str_radix(&b, 2).unwrap());
            assert_eq!(add_binary(&a, &b), want, "{a}+{b}");
            assert_eq!(par_add_binary(&a, &b), want, "{a}+{b}");
        }
    }

    #[test]
    fn random_long_inputs_parallel_equals_sequential() {
        let mut rng = rand::thread_rng();
        for _ in 0..300 {
            let (la, lb) = (rng.gen_range(1..=5000), rng.gen_range(1..=5000));
            let a = remove_leading_zeros(generate_random_binary(la));
            let b = remove_leading_zeros(generate_random_binary(lb));
            assert_eq!(par_add_binary(&a, &b), add_binary(&a, &b));
        }
    }
}
