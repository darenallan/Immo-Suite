//! Prototype (d) : import d'un Excel réaliste. Protocole : spikes/import/NOTE.md. Aucune donnée réelle.
//!
//!   spike-import analyze <fichier>          rapport canonique (JSON) sur la sortie standard
//!   spike-import bench <fichier> <passes>   durée de l'analyse (ms) : médiane et maximum
//!   spike-import db <fichier>               garanties de processus sur PostgreSQL (voir db.rs)

mod analyze;
mod db;

use std::path::Path;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("analyze") => {
            let report = analyze::analyze(Path::new(&a[2]));
            println!("{}", report.canonical());
        }
        Some("bench") => {
            let n: usize = a.get(3).and_then(|x| x.parse().ok()).unwrap_or(10);
            let mut t: Vec<f64> = (0..n)
                .map(|_| {
                    let s = Instant::now();
                    let r = analyze::analyze(Path::new(&a[2]));
                    std::hint::black_box(r.creatable());
                    s.elapsed().as_secs_f64() * 1000.0
                })
                .collect();
            t.sort_by(|x, y| x.partial_cmp(y).unwrap());
            println!("{:.1} {:.1}", t[t.len() / 2], t[t.len() - 1]);
        }
        Some("db") => {
            if let Err(e) = db::scenario(Path::new(&a[2]), a.get(3).map(String::as_str).unwrap_or("local")).await {
                eprintln!("ÉCHEC : {e}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("usage : spike-import analyze <fichier> | bench <fichier> <passes> | db <fichier> [local|neon]");
            std::process::exit(2);
        }
    }
}
