// dictionary zips are made up of json files
// this crate lets us convert that json data into struct instances
use serde::Deserialize;

use std::path::Path;

// Deserialize TRAIT needs to be implemented on STRUCT DictIndex
// impl Deserialize for DictIndex {} etc
// using the derive attribute here. this logic can be auto generated

// dictionary metadata. Filename and format
#[derive(Deserialize)]
pub struct DictIndex {
    pub title: String, // dictionary name
    pub format: u8,    // dictionary format. can be 1,2,3. thats why type u8
}

// searchable ja word in the dic. 食べる has fields たべる, score, definition, etc
#[derive(Deserialize)] // same thing as above
pub struct Term {
    pub word: String,                  // 食べる
    pub reading: String,               // たべる
    pub score: i32,                    // 550
    pub definition: serde_json::Value, // raw json from term bank. needs to be extracted later
}

// pub so main.rs can call load_dict_index. module items are by default private
// this fn takes a path to a zip. returns the metadata for that dictionary (title and format)
// 1 param. Path to the zip. Only borrows this data. Only needs to read not modify
// we wrap the return value in Result<>. Result is an enum with two variants. Ok(T) or Err(E). So either we got the metadata no issues or we encountered a problem
// the Ok(T) return value is DictIndex which is a struct with the metadata as fields
// the Err(E) return case is wrapped in a Box<>. Box is a smart pointer. it is fixed size (just a pointer). but it points to the heap which is needed because a variety of errs could occur here. of varying sizes. so we point at the heap to accomodate
// dyn is a keyword or qualifier. We could encounter a variety of errors. wrong format, no data, corrupt data, etc. We don't know which type it will be until runtime. dyn says i don't care which it is as long as it satisfies the Error trait from std. Figure it out at runtime and thats a valid return
pub fn load_dict_index(path: &Path) -> Result<DictIndex, Box<dyn std::error::Error>> {
    // let file = std::fs::File::open(path)?; is equivalent but we will be verbose for now
    let file = match std::fs::File::open(path) {
        Ok(f) => f,                     // on successful open. f is type struct File
        Err(e) => return Err(e.into()), // on unsuccessful open. we have error e, type struct Error from std::io. into method runs on e and converts it to our expected return error type. Box<dyn std::error::Error>
    };

    // by_name changes the "cursor" for where archive is which is why it needs to be mut. index and file do not they only read
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a, // on success. archive is a handle for the contents of the zip. we can now access json files etc
        Err(e) => return Err(e.into()), // here we convert to Box<dyn Error> like above. in this case from zip::result::ZipError instead of std::io::Error but into method handles it the same
    };

    // readable handle to index.json inside the zip
    let index = match archive.by_name("index.json") {
        Ok(i) => i,
        Err(e) => return Err(e.into()),
    };

    // deserialize index (json data) into dict_index (metadata struct)
    let dict_index = match serde_json::from_reader(index) {
        Ok(d) => d,
        Err(e) => return Err(e.into()),
    };

    Ok(dict_index) // we return the metadata struct but wrapped in the Ok enum variant (if successful)
}
