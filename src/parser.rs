use serde::Deserialize;
use zip::ZipArchive;
use std::{fs::File, path::Path};

// dictionary metadata. Filename and format
#[derive(Deserialize)]
pub struct DictIndex {
    pub title: String, // dictionary name
    pub format: u8,    // dictionary format. can be 1,2,3. thats why type u8
}

// searchable ja word in the dic. 食べる has fields たべる, score, definition, etc
#[derive(Deserialize)] // same thing as above
pub struct TermEntry {
    pub term: String,                  // 食べる
    pub reading: Option<String>,       // たべる
    pub score: i64, // 550. 64 to handle potentially big scores plus matches serde json integer convention
    pub definition: serde_json::Value, // raw json from term bank. needs to be extracted later
}

pub fn open_archive(path: &Path) -> Result<ZipArchive<File>, Box<dyn std::error::Error>> {
    // let file = std::fs::File::open(path)?; is equivalent but we will be verbose for now
    let file = match std::fs::File::open(path) {
        Ok(f) => f,                     // on successful open. f is type struct File
        Err(e) => return Err(e.into()), // on unsuccessful open. we have error e, type struct Error from std::io. into method runs on e and converts it to our expected return error type. Box<dyn std::error::Error>
    };

    let archive = match zip::ZipArchive::new(file) {
        Ok(f) => f,
        Err(e) => return Err(e.into()),
    };

    Ok(archive)
}

pub fn load_dict_index(
    archive: &mut ZipArchive<File>,
) -> Result<DictIndex, Box<dyn std::error::Error>> {
    let index = match archive.by_name("index.json") {
        Ok(i) => i,
        Err(e) => return Err(e.into()),
    };

    let dict_index = match serde_json::from_reader(index) {
        Ok(d) => d,
        Err(e) => return Err(e.into()),
    };

    Ok(dict_index)
}

pub fn load_terms(
    archive: &mut ZipArchive<File>,
) -> Result<Vec<TermEntry>, Box<dyn std::error::Error>> {
    let terms = match archive.by_name("term_bank_1.json") {
        Ok(t) => t,
        Err(e) => return Err(e.into()),
    };

    let term_entry: Vec<(
        String,
        Option<String>,
        Option<String>,
        String,
        i64,
        serde_json::Value,
        i64,
        String,
    )> = match serde_json::from_reader(terms) {
        Ok(t) => t,
        Err(e) => return Err(e.into()),
    };

    Ok(term_entry
        .into_iter() //instead of for loop
        .map(|t| TermEntry {
            // closure so we dont need to write a seperate fn to handle TermEntry fields from the serde_json we can jsut do inline
            term: t.0,
            reading: t.1,
            score: t.4,
            definition: t.5,
        })
        .collect()) // make a vec of TermEntry's
}
