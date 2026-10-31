//! `spike-amount <n>` : imprime le montant en lettres capitalisé (utilisé par make_cases.py).
fn main() {
    let n: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).expect("usage : spike-amount <entier>");
    println!("{}", spike_words::amount_in_words_capitalized(n));
}
