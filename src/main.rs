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

    // Third: 64 digits per word
    let start = Instant::now();
    let words_result = par_add_binary_words(&binary_number1, &binary_number2);
    println!("2. Parallel sum 3 (64-digit words) took {:?}", start.elapsed());
    assert_eq!(words_result, seq_result, "par_add_binary_words gives a wrong sum");

    // Fourth: parallel prefix (slow on long inputs, so only up to 100,000 digits)
    if len <= 100_000 {
        let start = Instant::now();
        let prefix_result = par_add_binary_prefix(&binary_number1, &binary_number2);
        println!("3. Parallel sum 4 (prefix) took {:?}", start.elapsed());
        assert_eq!(prefix_result, seq_result, "par_add_binary_prefix gives a wrong sum");
    } else {
        println!("3. Parallel sum 4 (prefix) skipped (slow above 100,000 digits)");
    }

    println!("4. Sequential sum took {:?}", seq_time);
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

// Third algo (fast): 64 digits per u64 word
//
// A u64 holds 64 binary digits, so one `overflowing_add` adds 64 digits at once and
// reports the carry. The digits are packed into words, the words are added, and the
// result is unpacked again. For short inputs (one word) this is just one machine add.
// For long inputs the words are split into chunks, one per thread:
//   1. every chunk adds its words with carry-in 0 and remembers (carry-out, "all ones?")
//   2. the carries are chained chunk by chunk (a handful of steps)
//   3. chunks that really received a carry-in add 1 (stops at the first word that does not overflow)
fn pack_word(bytes: &[u8], w: usize) -> u64 {
    // word w = digits [64w .. 64w+64) counted from the right
    let hi = bytes.len().saturating_sub(64 * w);
    let lo = hi.saturating_sub(64);
    bytes[lo..hi].iter().fold(0u64, |x, &c| (x << 1) | (c - b'0') as u64)
}

fn pack(s: &str, words: usize) -> Vec<u64> {
    let b = s.as_bytes();
    (0..words).into_par_iter().map(|w| pack_word(b, w)).collect()
}

fn add_words_par(a: &[u64], b: &[u64]) -> (Vec<u64>, bool) {
    let n = a.len();
    let threads = rayon::current_num_threads();
    let chunk = ((n + threads - 1) / threads).max(1);
    let mut out = vec![0u64; n];
    let info: Vec<(bool, bool)> = out
        .par_chunks_mut(chunk)
        .zip(a.par_chunks(chunk))
        .zip(b.par_chunks(chunk))
        .map(|((o, a), b)| {
            let (mut carry, mut all_ones) = (false, true);
            for i in 0..o.len() {
                let (s1, o1) = a[i].overflowing_add(b[i]);
                let (s2, o2) = s1.overflowing_add(carry as u64);
                o[i] = s2;
                carry = o1 || o2;
                all_ones &= s2 == u64::MAX;
            }
            (carry, all_ones)
        })
        .collect();
    let mut carry_in = vec![false; info.len()];
    let mut carry = false;
    for (k, &(carry_out, all_ones)) in info.iter().enumerate() {
        carry_in[k] = carry;
        carry = carry_out || (carry && all_ones);
    }
    out.par_chunks_mut(chunk).zip(carry_in.par_iter()).for_each(|(o, &ci)| {
        if ci {
            for w in o.iter_mut() {
                let (s, overflow) = w.overflowing_add(1);
                *w = s;
                if !overflow {
                    break;
                }
            }
        }
    });
    (out, carry)
}

fn unpack(words: &[u64], carry: bool) -> String {
    let mut w = words.to_vec();
    if carry {
        w.push(1);
    }
    let n = w.len();
    let mut out = vec![0u8; 64 * n];
    out.par_chunks_mut(64).enumerate().for_each(|(c, chunk)| {
        let x = w[n - 1 - c]; // most significant word first
        for j in 0..64 {
            chunk[j] = b'0' + ((x >> (63 - j)) & 1) as u8;
        }
    });
    remove_leading_zeros(String::from_utf8(out).unwrap())
}

fn par_add_binary_words(a: &str, b: &str) -> String {
    let words = ((a.len().max(b.len()) + 63) / 64).max(1);
    let (sum, carry) = add_words_par(&pack(a, words), &pack(b, words));
    unpack(&sum, carry)
}

// Fourth algo: parallel prefix (carry-lookahead)
//
// Parallel at EVERY digit, so it also works for 1 digit + 1 digit.
// For each digit i (least significant first):
//   g = a AND b  -> this digit "generates" a carry by itself
//   p = a XOR b  -> this digit "passes" an incoming carry on
// Two neighbouring groups combine as  G = g_high OR (p_high AND g_low),  P = p_high AND p_low.
// This is associative, so after log2(n) rounds (each digit looks 1, 2, 4, ... places to its
// right) G[i] is the carry that comes OUT of digit i. Sum digit i = p[i] XOR G[i-1].
// It does much more work than the other algorithms (n * log2 n), so it is a teaching
// example, not a fast one on 4 cores.
fn par_add_binary_prefix(a: &str, b: &str) -> String {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let n = a.len().max(b.len());
    let digit = |s: &[u8], i: usize| if i < s.len() { s[s.len() - 1 - i] - b'0' } else { 0 };
    let p0: Vec<u8> = (0..n).into_par_iter().map(|i| digit(a, i) ^ digit(b, i)).collect();
    let mut g: Vec<u8> = (0..n).into_par_iter().map(|i| digit(a, i) & digit(b, i)).collect();
    let mut p = p0.clone();
    let mut d = 1;
    while d < n {
        let (ng, np): (Vec<u8>, Vec<u8>) = (0..n)
            .into_par_iter()
            .map(|i| {
                if i >= d {
                    (g[i] | (p[i] & g[i - d]), p[i] & p[i - d])
                } else {
                    (g[i], p[i])
                }
            })
            .unzip();
        g = ng;
        p = np;
        d *= 2;
    }
    let mut out: Vec<u8> = (0..n)
        .into_par_iter()
        .map(|i| b'0' + (p0[i] ^ if i == 0 { 0 } else { g[i - 1] }))
        .collect();
    if g[n - 1] == 1 {
        out.push(b'1');
    }
    out.reverse();
    remove_leading_zeros(String::from_utf8(out).unwrap())
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

    #[test]
    fn words_and_prefix_edge_cases() {
        for (a, b, want) in [("0", "0", "0"), ("1", "0", "1"), ("1", "1", "10"), ("11", "01", "100"), ("1111", "1", "10000")] {
            assert_eq!(par_add_binary_words(a, b), want, "{a}+{b}");
            assert_eq!(par_add_binary_prefix(a, b), want, "{a}+{b}");
        }
    }

    #[test]
    fn words_carry_across_word_and_chunk_borders() {
        for n in [63, 64, 65, 127, 128, 129, 1000, 4096, 10_000] {
            let a = "1".repeat(n);
            let want = format!("1{}", "0".repeat(n));
            assert_eq!(par_add_binary_words(&a, "1"), want, "n={n}");
            assert_eq!(par_add_binary_prefix(&a, "1"), want, "n={n}");
        }
    }

    #[test]
    fn words_and_prefix_match_sequential() {
        let mut rng = rand::thread_rng();
        for _ in 0..500 {
            let (la, lb) = (rng.gen_range(1..=5000), rng.gen_range(1..=5000));
            let a = remove_leading_zeros(generate_random_binary(la));
            let b = remove_leading_zeros(generate_random_binary(lb));
            let want = add_binary(&a, &b);
            assert_eq!(par_add_binary_words(&a, &b), want);
            assert_eq!(par_add_binary_prefix(&a, &b), want);
        }
    }
}
