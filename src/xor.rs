//!

use std::collections::HashMap;
use std::str;
use std::f32;
use string_utils::{byte_array_to_hex, hex_to_byte_array, bytes_to_ascii_string};

///
/// Generates an xor'd hex encoding of two hex strings
///
/// # Panics
/// - If `s1` is not the same length as `s2`
/// - If `s1` or `s2` contains non-hexadecimal characters
///
pub fn hex_fixed_xor(s1: &String, s2: &String) -> String {
    if s1.len() != s2.len() {
        panic!("Input strings must be the same length");
    }

    byte_array_to_hex(&fixed_xor(&hex_to_byte_array(s1), &hex_to_byte_array(s2)))
}

fn fixed_xor(v1: &Vec<u8>, v2: &Vec<u8>) -> Vec<u8> {
    if v1.len() != v2.len() {
        panic!("Input vectors must be the same length");
    }

    let mut v = Vec::new();
    let mut index = 0;
    while index < v1.len() {
        v.push(v1[index] ^ v2[index]);
        index += 1;
    }

    v
}

pub fn xor_cypher_decrypt_char_frequency(s: &String) -> (String, f32) {
    let bytes = hex_to_byte_array(s);
    let mut min_score = f32::INFINITY;
    let mut best_candidate = "".to_string();

    for key in 0..255 as u8 {
        let cypher = vec![key; bytes.len()];
        let cleartext_candidate = fixed_xor(&bytes, &cypher);
        let ascii_string = match bytes_to_ascii_string(&cleartext_candidate) {
            Some(s) => s,
            None => {
                continue;
            }
        };
        let score = score_candidate(&ascii_string);
        if score < min_score {
            min_score = score;
            best_candidate = ascii_string;
        }
    }

    (best_candidate, min_score)
}

fn score_candidate(s: &String) -> f32 {
    let mut map: HashMap<char, u8> = HashMap::new();

    // https://www.math.cornell.edu/~mec/2003-2004/cryptography/subs/frequencies.html
    let corpus_frequency_data: [(char, f32); 26] = [
        ('a', 0.0812),
        ('b', 0.0149),
        ('c', 0.0271),
        ('d', 0.0432),
        ('e', 0.1202),
        ('f', 0.0230),
        ('g', 0.0203),
        ('h', 0.0592),
        ('i', 0.0731),
        ('j', 0.0010),
        ('k', 0.0069),
        ('l', 0.0398),
        ('m', 0.0261),
        ('n', 0.0695),
        ('o', 0.0768),
        ('p', 0.0182),
        ('q', 0.0011),
        ('r', 0.0602),
        ('s', 0.0628),
        ('t', 0.0910),
        ('u', 0.0288),
        ('v', 0.0111),
        ('w', 0.0209),
        ('x', 0.0017),
        ('y', 0.0211),
        ('z', 0.0007),
    ];

    let stripped_string = s.replace(" ", "");

    for c in s.replace(" ", "").to_lowercase().chars() {
        let count = map.entry(c).or_insert(0);
        *count += 1;
    }

    let mut score = 0.0;
    for &(c, corpus_frequency) in corpus_frequency_data.iter() {
        let letter_frequency = *map.get(&c).unwrap_or(&0) as f32 / stripped_string.len() as f32;
        let letter_score = ((letter_frequency * 100.0 + 1.0).log(2.0) -
                                (corpus_frequency * 100.0 + 1.0).log(2.0))
            .abs();
        score += letter_score;
    }

    let mut modifier = 2.0;
    for c in s.chars() {
        if (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == ' ' || c == '.' || c == '\'' {
            modifier *= 1.15;
        } else {
            modifier /= 1.2;
        }
    }
    score -= modifier;
    score += 1.0 / modifier;

    score
}

pub fn detect_single_char_xor(v: &Vec<&str>) -> (usize, String) {
    let mut min_score = f32::INFINITY;
    let mut best_cleartext = "".to_string();
    let mut best_index = 0;

    for (index, s) in v.iter().enumerate() {
        let (best_decoding, score) = xor_cypher_decrypt_char_frequency(&s.to_string());
        if score < min_score {
            min_score = score;
            best_cleartext = best_decoding;
            best_index = index;
        }
    }

    (best_index, best_cleartext)
}

pub fn repeating_key_xor(s: &String, key: &String) -> String {
    let bytes = s.clone().into_bytes();
    let key_bytes = key.clone().into_bytes();
    let mut cypher = Vec::new();

    for i in 0..bytes.len() {
        cypher.push(key_bytes[i % key_bytes.len()]);
    }
    byte_array_to_hex(&fixed_xor(&bytes, &cypher))
}


#[cfg(test)]
mod tests {
    mod hex_fixed_xor {
        use super::super::hex_fixed_xor;

        #[test]
        fn it_xors_empty_strings() {
            let hex = "".to_string();
            assert_eq!(hex_fixed_xor(&hex, &hex), hex);
        }

        #[test]
        #[should_panic]
        fn it_panics_on_odd_length_strings() {
            let hex = "4ac93".to_string();
            hex_fixed_xor(&hex, &hex);
        }

        #[test]
        #[should_panic]
        fn it_panics_on_non_hex_characters() {
            let hex = "4ag9".to_string();
            hex_fixed_xor(&hex, &hex);
        }

        #[test]
        fn it_xors_hex_strings() {
            let hex1 = "1c0111001f010100061a024b53535009181c".to_string();
            let hex2 = "686974207468652062756c6c277320657965".to_string();
            let expected = "746865206b696420646f6e277420706c6179".to_string();
            assert_eq!(hex_fixed_xor(&hex1, &hex2), expected);
        }
    }

    mod xor_cypher_decrypt_char_frequency {
        use super::super::xor_cypher_decrypt_char_frequency;

        #[test]
        fn it_solves_the_example() {
            let hex = "1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736"
                .to_string();
            let expected = "Cooking MC's like a pound of bacon".to_string();
            let (result, _) = xor_cypher_decrypt_char_frequency(&hex);
            assert_eq!(result, expected);
        }
    }

    mod repeating_key_xor {
        use super::super::repeating_key_xor;

        #[test]
        fn it_solves_the_example() {
            let input = "Burning 'em, if you ain't quick and nimble\nI go crazy when I hear a cymbal"
                .to_string();
            let key = "ICE".to_string();
            let expected = "0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f".to_string();
            assert_eq!(repeating_key_xor(&input, &key), expected);
        }
    }
}
