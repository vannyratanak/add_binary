use rayon::prelude::*;
use rand::prelude::*;

fn main() {
    // Generate random binary numbers
    let len = rand::thread_rng().gen_range(1..=10);
    let binary_number1 = remove_leading_zeros(generate_random_binary(len));
    let binary_number2 = remove_leading_zeros(generate_random_binary(len));

    // Print generated binary numbers
    println!("Binary1: {}", binary_number1);
    println!("Binary2: {}", binary_number2);

    // // Parallel sum 1
    // // **** noted that this algorithmn work only length of binary below 64
    let start = std::time::Instant::now();
    let parallel_result = par_add_binary1(binary_number1.clone(), binary_number2.clone());
    println!("\n0. Parallel sum: {}", parallel_result);
    println!("Parallel 1 took {:?}", start.elapsed());

    // Parallel sum
    let start = std::time::Instant::now();
    let parallel_result = par_add_binary(binary_number1.clone(), binary_number2.clone());
    println!("\n1. Parallel sum: {}", parallel_result);
    println!("Parallel 2 took {:?}", start.elapsed());

    // Sequential sum
    let start = std::time::Instant::now();
    let seq_result = add_binary(binary_number1.to_string(), binary_number2.to_string());
    println!("\n2. Sequence sum: {}", seq_result);
    println!("Sequence took {:?}", start.elapsed());
}

// Sequential sum binary
fn add_binary(a: String, b: String) -> String {
    let a = a.chars().rev().collect::<String>();
    let b = b.chars().rev().collect::<String>();
    let mut result = String::new();
    let mut carry = 0;
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let digit_a = a.chars().nth(i).unwrap_or('0').to_digit(2).unwrap_or(0);
        let digit_b = b.chars().nth(i).unwrap_or('0').to_digit(2).unwrap_or(0);
        let sum = digit_a + digit_b + carry;
        result.push_str(&(sum % 2).to_string());
        carry = sum / 2;
        // println!("Index {}, digit_a: {}, digit_b: {}, sum: {}, carry: {}", i, digit_a, digit_b, sum, carry);
    }

    if carry == 1 {
        result.push('1');
    }
    
    let result_str = result.chars().rev().skip_while(|&c| c == '0').collect::<String>();

    // If the result is '0' and there's only one digit, return '0'
    if result_str == "" || result_str.len() == 0 {
        "0".to_string()
    } else {
        result_str
    }
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
fn par_add_binary1(a: String, b: String) -> String {
    // Convert binary strings to decimals in parallel
    let sum_decimal: u64 = vec![a.clone(), b.clone()]
        .par_iter()
        .map(|binary| binary_to_decimal(binary.to_string()))
        .sum();

    // Convert the sum decimal to binary
    decimal_to_binary(sum_decimal)
}

fn binary_to_decimal(binary: String) -> u64 {
    u64::from_str_radix(&binary, 2).expect("Invalid binary number")
}

fn decimal_to_binary(decimal: u64) -> String {
    format!("{:b}", decimal)
}

// Second algo of parallel sum binary
fn par_add_binary(a: String, b: String) -> String {
    let a = a.chars().rev().collect::<String>();
    let b = b.chars().rev().collect::<String>();
    let max_len = a.len().max(b.len());

    let chunk_size = (max_len as f64 / rayon::current_num_threads() as f64).ceil() as usize;

    let result_chunks: Vec<String> = (0..max_len)
        .into_par_iter()
        .chunks(chunk_size)
        .map(|chunk| {
            let a_clone = a.clone(); 
            let b_clone = b.clone(); 
            let indices = chunk.clone();

            let mut carry=0;

            let chunk_result: String = chunk.into_iter().zip(indices).map(|(_, i)| {
                let digit_a = a_clone.chars().nth(i).unwrap_or('0').to_digit(2).unwrap_or(0);
                let digit_b = b_clone.chars().nth(i).unwrap_or('0').to_digit(2).unwrap_or(0);
                // println!("\nI[{}]: carry: {}, a: {}, b: {}",i,carry,digit_a,digit_b);                
                let sum = digit_a + digit_b + carry;
                if sum == 3 {
                    let result = 1;
                    carry = 1;                    
                    // println!("carry[{}]: {}",i,carry);
                    // println!("result[{}]: {}",i ,result);
                    result.to_string()
                } else if sum == 2 {
                    let result = 0;
                    carry = 1;
                    // println!("carry[{}]: {}",i,carry);
                    // println!("result[{}]: {}",i ,result);
                    result.to_string()                          
                } else if sum == 1 {
                    let result = 1;
                    carry = 0;
                    // println!("carry[{}]: {}",i,carry);
                    // println!("result[{}]: {}",i ,result);
                    result.to_string()                          
                } else if sum == 0 {
                    let result = 0;
                    carry = 0;
                    // println!("carry[{}]: {}",i,carry);
                    // println!("result[{}]: {}",i ,result);
                    result.to_string()                          
                
                } else {
                    let result = sum % 2;
                    carry = sum / 2;
                    // println!("carry[{}]: {}",i,carry);
                    // println!("result[{}]: {}",i ,result);
                    result.to_string()                 
                }

            }).collect();

            let mut chunk_result = chunk_result.chars().collect::<String>();
            
            if carry > 0 {
                // println!("carry:{}",carry);
                chunk_result.insert(0, '1');
            }

            chunk_result
        })
        .collect();

    let result = result_chunks.into_iter().rev().collect::<String>();

    result
}
