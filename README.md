# Parallel Programming Report

## 1. Overview

Binary addition is a fundamental operation often used in computing, and optimizing its performance can be crucial. The provided Rust program explores both sequential and parallel algorithms for binary addition using the rayon library to harness parallel processing capabilities.

## 2. Program Structure

### 3. Binary Addition Functions

#### 1. Sequential Binary Addition

- **Function:** `add_binary(a: String, b: String) -> String`
- **Description:** This function sequentially performs binary addition. It reverts values `a` and `b` from left to right and then calculates each index following the binary sum method.

  - Ex: a = '111', b = '110'
  - rev_a= '101', rev_b = '011'
  - index 1: a+b = 1+0 = 1, carry 0
  - index 2: a+b = 0+1 = 1, carry 0
  - index 3: a+b = 1+1 = 0, carry 1

  - The output will be `1101`

#### 2. First Parallel Binary Addition

- **Function:** `par_add_binary1(a: String, b: String) -> String`
- **Description:** In this first parallel binary addition algorithm, I convert binary strings to decimals in parallel, add them concurrently, and convert the sum back to binary using the rayon library.
- **Problem:** There is a memory overflow problem when the length of variables `a` and `b` reach 100.

#### 3. Second Parallel Binary Addition

- **Function:** `par_add_binary(a: String, b: String) -> String`
- **Description:** The second parallel binary addition algorithm divides the binary addition into chunks and performs addition in parallel. It utilizes the rayon library to parallelize the process efficiently, handling carry and other binary addition logic.
- **Problem:** When using parallel processing, it splits into two parts, and handling the "carry" value becomes challenging. Some conditions are successfully handled, such as `1 + 0 = 1` with carry 0, and `0 + 0 = 0` with carry 0.

### 4. Binary Conversion Functions

- **Function:** `binary_to_decimal(binary: String) -> u64`
- **Description:** Converts a binary string to a decimal number.

- **Function:** `decimal_to_binary(decimal: u64) -> String`
- **Description:** Converts a decimal number to a binary string.

### 5. Random Binary Generation

- **Function:** `generate_random_binary(length: usize) -> String`
- **Description:** Generates a random binary string of the specified length using the rand crate.

## 4. Main Function

- Generates two random binary numbers of varying lengths.
- Performs sequential binary addition and measures the execution time.
- Performs the first parallel binary addition and measures the execution time.
- Performs the second parallel binary addition and measures the execution time.

## 5. Output

The program prints the execution time for both sequential and parallel binary addition. It is noteworthy that the first parallel algorithm `par_add_binary1` is designed to work for binary numbers with a length below 64.

## 6. Analysis

- Conducted experiments on the length index of variable `a = 5` (e.g., 11111) and `b = 5` (e.g., 10101).
  
  ![alt text](https://github.com/vannyratanak/add_binary/blob/main/img/capture.png)
  
  The first algorithm took more time than the other two algorithms.
- Conducted experiments on the length index of variable `a = 64` (64 digits) and `b = 64` (64 digits).
  
  ![alt text](https://github.com/vannyratanak/add_binary/blob/main/img/capture1.png)

  The execution times of the second parallel algorithm `par_add_binary` and the sequential algorithm are quite similar.

- Conducted experiments on the length index of variable `a = 100_000` (100,000 digits) and `b = 100_000` (100,000 digits).
  
  ![alt text](https://github.com/vannyratanak/add_binary/blob/main/img/capture2.png)

   The larger the data given, the execution time of the second parallel algorithm `par_add_binary` is faster compared to the sequential algorithm. However, the first algorithm `par_add_binary1` isn’t executable due to the given data being very large.

## 7. Conclusion

The program is not fully successful in demonstrating parallel binary addition; it needs more improvements. However, further testing and refinement may be required to ensure the robustness and scalability of the parallel algorithms, especially for larger binary numbers.
