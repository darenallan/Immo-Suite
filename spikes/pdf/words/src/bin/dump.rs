//! Exporte `n<TAB>texte` pour 0..=2000, les frontières, 999 999 999 et un échantillon fixe de 20 000 valeurs,
//! pour les contrôles indépendants en Python (`spikes/pdf/check_words.py`).
use spike_words::number_in_words;

fn main() {
    let mut values: Vec<u64> = (0..=2000).collect();
    values.extend([2_000_000, 21_000_000, 80_000_000, 200_000_000, 999_999_999, 1_000_000, 1_250_000, 80_000, 100_000, 200_000]);
    let mut x: u64 = 0x2545_F491_4F6C_DD1D;
    for _ in 0..20_000 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        values.push(x % 1_000_000_000);
    }
    for n in values {
        println!("{n}\t{}", number_in_words(n));
    }
}
