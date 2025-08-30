use std::time::Instant;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use rayon::prelude::*;
use memmap2::Mmap;
use std::fmt::Write;

fn main() {
    let now = Instant::now(); // for benchmarking time

    let mut file = File::open("input.txt").unwrap();


    // ~233000 ms
    // let mut text = String::new();
    // file.read_to_string(&mut text).unwrap();
    // text = text.to_lowercase();
    // let letters: [&str; 26] = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];
    // let mut letter_counts: [i32; 26] = [0; 26];
    // for i in 0..26 {
    //     for letter in text.chars() {
    //         if letter.to_string() == letters[i] {
    //             letter_counts[i] += 1;
    //         }
    //     }
    // }
    // println!("Letter counts:");
    // for i  in 0..26 {
    //     println!("{}: {}", letters[i], letter_counts[i]);
    // }


    // ~21400ms
    // let mut text = String::new();
    // file.read_to_string(&mut text).unwrap();
    // text = text.to_lowercase();
    // let letters = "abcdefghijklmnopqrstuvwxyz";
    // let mut letter_counts = HashMap::new();
    // for c in letters.chars() {
    //     let mut temp = 0;
    //     for letter in text.chars() {
    //         if letter == c {
    //             temp += 1;
    //         }
    //     }
    //     if temp != 0 {
    //         letter_counts.insert(c, temp);
    //     }
    // }
    // println!("Letter counts:");
    // for c in letters.chars() {
    //     if let Some(count) = letter_counts.get(&c) {
    //         println!("{}: {}", c, count);
    //     }
    // }


    // ~15500ms
    // let mut text = String::new();
    // file.read_to_string(&mut text).unwrap();
    // text = text.to_lowercase();
    // let mut letter_counts = HashMap::from([
    //     ('a', 0), ('b', 0), ('c', 0), ('d', 0), ('e', 0), ('f', 0), 
    //     ('g', 0), ('h', 0), ('i', 0), ('j', 0), ('k', 0), ('l', 0), 
    //     ('m', 0), ('n', 0), ('o', 0), ('p', 0), ('q', 0), ('r', 0), 
    //     ('s', 0), ('t', 0), ('u', 0), ('v', 0), ('w', 0), ('x', 0), 
    //     ('y', 0), ('z', 0),
    // ]);
    // for letter in text.chars() {
    //     if let Some(v) = letter_counts.get_mut(&letter) {
    //         *v += 1;
    //     }
    // }
    // let result: String = "abcdefghijklmnopqrstuvwxyz"
    // .chars()
    // .filter_map(|c| {
    //     letter_counts.get(&c).and_then(|&count| {
    //         if count != 0 {
    //             Some(format!("{}: {}\n", c, count))
    //         } else {
    //             None
    //         }
    //     })
    // })
    // .collect();
    // print!("{}", result);


    // ~11700ms
    // let mut text = String::new();
    // file.read_to_string(&mut text).unwrap();
    // let mut letter_counts: HashMap<char, usize> = HashMap::with_capacity(26);
    // for byte in text.bytes() {
    //     if byte.is_ascii_alphabetic() {
    //         let c = (byte as char).to_ascii_lowercase();
    //         *letter_counts.entry(c).or_insert(0) += 1;
    //     }
    // }
    // let mut result = String::with_capacity(52);
    // for c in 'a'..='z' {
    //     if let Some(&count) = letter_counts.get(&c) {
    //         if count > 0 {
    //             result.push_str(&format!("{}: {}\n", c, count));
    //         }
    //     }
    // }
    // print!("{}", result);


    // ~1520ms
    // let mut text = String::new();
    // file.read_to_string(&mut text).unwrap();
    // let mut letter_counts = [0usize; 26];
    // for byte in text.bytes() {
    //     if byte.is_ascii_alphabetic() {
    //         let idx = (byte.to_ascii_lowercase() - b'a') as usize;
    //         letter_counts[idx] += 1;
    //     }
    // }
    // let mut result = String::with_capacity(52);
    // for (i, &count) in letter_counts.iter().enumerate() {
    //     if count > 0 {
    //         let letter = (b'a' + i as u8) as char;
    //         result.push_str(&format!("{}: {}\n", letter, count));
    //     }
    // }
    // print!("{}", result);

    // ~750ms
    // let mut text = String::new();
    // file.read_to_string(&mut text).unwrap();
    // let letter_counts = text
    //     .par_bytes()
    //     .filter(|&byte| byte.is_ascii_alphabetic())
    //     .map(|byte| (byte.to_ascii_lowercase() - b'a') as usize)
    //     .fold(|| [0usize; 26], |mut acc, idx| {
    //         acc[idx] += 1;
    //         acc
    //     })
    //     .reduce(|| [0usize; 26], |mut acc1, acc2| {
    //         for i in 0..26 {
    //             acc1[i] += acc2[i];
    //         }
    //         acc1
    //     }); // Reduce and combine all the results
    // let mut result = String::with_capacity(52);
    // for (i, &count) in letter_counts.iter().enumerate() {
    //     if count > 0 {
    //         let letter = (b'a' + i as u8) as char;
    //         result.push_str(&format!("{}: {}\n", letter, count));
    //     }
    // }
    // print!("{}", result);

    // 105ms
    // let mmap = unsafe { Mmap::map(&file).unwrap() };
    // let chunk_size = 65536;
    // let letter_counts = mmap
    //     .par_chunks(chunk_size)
    //     .map(|chunk| {
    //         let mut local_counts = [0usize; 26];
    //         for &byte in chunk {
    //             if byte.is_ascii_alphabetic() {
    //                 let idx = (byte.to_ascii_lowercase() - b'a') as usize;
    //                 local_counts[idx] += 1;
    //             }
    //         }
    //         local_counts
    //     })
    //     .reduce(|| [0usize; 26], |mut acc1, acc2| {
    //         for i in 0..26 {
    //             acc1[i] += acc2[i];
    //         }
    //         acc1
    //     });
    // let mut result = String::with_capacity(52);
    // for (i, &count) in letter_counts.iter().enumerate() {
    //     if count > 0 {
    //         let letter = (b'a' + i as u8) as char;
    //         result.push_str(&format!("{}: {}\n", letter, count));
    //     }
    // }
    // print!("{}", result);

    
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let chunk_size = 16 * 1024;
    let letter_counts = mmap
        .par_chunks(chunk_size)
        .fold(|| [0usize; 26], #[inline(always)] |mut local_counts, chunk| {
            for &byte in chunk {
                if byte.is_ascii_alphabetic() {
                    let idx = (byte | 0x20) as usize - b'a' as usize;
                    local_counts[idx] += 1;
                }
            }
            local_counts
        })
        .reduce(|| [0usize; 26], #[inline(always)] |mut acc1, acc2| {
            for i in 0..26 {
                acc1[i] += acc2[i];
            }
            acc1
        });

    let mut result = String::with_capacity(1024);
    for (i, &count) in letter_counts.iter().enumerate() {
        if count > 0 {
            let letter = (b'a' + i as u8) as char;
            write!(result, "{}: {}\n", letter, count).unwrap();
        }
    }
    print!("{}", result);


    let elapsed = now.elapsed();
    println!("Total time taken: {:.2?}", elapsed);
}
