//! E3 en bibliothèque : Typst compilé dans le processus (pas de démarrage de processus, polices chargées une fois).
//! Usage : spike-typst-lib <cas.json> <a4|a5> <sortie.pdf> [--repeat N]

use std::path::{Path, PathBuf};
use std::time::Instant;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Dict, IntoValue};
use typst_layout::PagedDocument;
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

struct Spike {
    root: PathBuf,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: FileId,
}

impl Spike {
    fn new(root: &Path, fonts: Vec<Font>, case: &str, fmt: &str) -> Self {
        let mut inputs = Dict::new();
        inputs.insert("case".into(), case.into_value());
        inputs.insert("fmt".into(), fmt.into_value());
        let library = Library::builder().with_inputs(inputs).build();
        let book = FontBook::from_fonts(fonts.iter());
        Spike {
            root: root.to_path_buf(),
            library: LazyHash::new(library),
            book: LazyHash::new(book),
            fonts,
            main: FileId::new(RootedPath::new(VirtualRoot::Project, VirtualPath::new("quittance.typ").unwrap())),
        }
    }
    fn read(&self, id: FileId) -> FileResult<Vec<u8>> {
        let path = id.vpath().resolve(&self.root).ok_or(FileError::AccessDenied)?;
        std::fs::read(&path).map_err(|e| FileError::from_io(e, &path))
    }
}

impl World for Spike {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        let text = String::from_utf8(self.read(id)?).map_err(|_| FileError::InvalidUtf8)?;
        Ok(Source::new(id, text))
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Ok(Bytes::new(self.read(id)?))
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        None
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let root = root.canonicalize().unwrap();
    let repeat: usize = a.iter().position(|x| x == "--repeat").and_then(|i| a.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(1);
    // chemin du cas relatif à la racine du prototype : « cases/<id>.json »
    let case_abs = PathBuf::from(&a[1]).canonicalize().unwrap();
    let case = case_abs.strip_prefix(root.canonicalize().unwrap()).unwrap().to_string_lossy().replace('\\', "/");

    let t_fonts = Instant::now();
    let mut fonts = vec![];
    for f in ["IBMPlexSans-Regular.ttf", "IBMPlexSans-Bold.ttf"] {
        let data = Bytes::new(std::fs::read(root.join("fonts/static").join(f)).unwrap());
        fonts.push(Font::new(data, 0).unwrap());
    }
    eprintln!("polices chargées en {:.1} ms (une fois)", t_fonts.elapsed().as_secs_f64() * 1000.0);

    let mut times = vec![];
    let mut pdf = vec![];
    for _ in 0..repeat {
        let t0 = Instant::now();
        let world = Spike::new(&root, fonts.clone(), &case, &a[2]);
        let doc = typst::compile::<PagedDocument>(&world);
        let doc = match doc.output {
            Ok(d) => d,
            Err(e) => {
                eprintln!("échec de compilation : {:?}", e.iter().map(|d| d.message.to_string()).collect::<Vec<_>>());
                std::process::exit(2);
            }
        };
        let options = typst_pdf::PdfOptions::default();
        pdf = typst_pdf::pdf(&doc, &options).unwrap_or_else(|e| {
            eprintln!("échec PDF : {:?}", e.iter().map(|d| d.message.to_string()).collect::<Vec<_>>());
            std::process::exit(2)
        });
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    std::fs::write(&a[3], &pdf).unwrap();
    if repeat > 1 {
        times.sort_by(|x, y| x.partial_cmp(y).unwrap());
        println!("{:.1} {:.1}", times[times.len() / 2], times[((times.len() - 1) as f64 * 0.95).round() as usize]);
    }
}
