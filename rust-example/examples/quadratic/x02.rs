const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];

fn syllable_estimate(word: &str) -> usize {
    let mut count = 0;
    let mut prev_vowel = false;
    for c in word.chars() {
        let lower = c.to_ascii_lowercase();
        let is_vowel = VOWELS.contains(&lower) || lower == 'y';
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }
    if word.ends_with('e') && count > 1 {
        count -= 1;
    }
    count.max(1)
}

fn main() {
    let text = "the quick brown fox jumped over seventeen lazy elephants yesterday";
    let mut total = 0;
    for word in text.split_whitespace() {
        let n = syllable_estimate(word);
        total += n;
        println!("{:<12} {}", word, n);
    }
    println!("total syllables: {}", total);
}
