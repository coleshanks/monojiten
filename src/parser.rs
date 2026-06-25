use serde::Deserialize;
use zip::ZipArchive;
use std::{fs::File, path::Path};

// dictionary metadata. title and format
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
        Ok(a) => a, // archive can be thought of like a pointer inside the zip. i now have access to the stuff here and can point to and access stuff. the jsons etc
        Err(e) => return Err(e.into()),
    };

    Ok(archive) // return this archive pointer
}

// take this archive pointer thing inside the zip. it needs to be mut because
// even tho we are just reading by_name modifies the internal cursor to seek to the right file
// returns our struct dict_index which is dictionary's metadata (title and format)
pub fn load_dict_index(
    archive: &mut ZipArchive<File>,
) -> Result<DictIndex, Box<dyn std::error::Error>> {
    let index = match archive.by_name("index.json") {
        Ok(i) => i, // input paramater archive is like our pointer inside the zip. index is now our pointer to index.json. its not yet the metadata inside that file it just lets us access it
        Err(e) => return Err(e.into()),
    };

    let dict_index = match serde_json::from_reader(index) {
        Ok(d) => d, // now we use index to unpack index.json data into dict_index struct
        Err(e) => return Err(e.into()),
    };

    Ok(dict_index)
}

// right now points at a specific term bank. to be updated later
// returns a Vec of TermEntrys. Fields from the json into our struct is deserialized and nice. Except definition which is complicated and can be many layers nested/ That stays raw json serde and we process it later in fn extract_definition
pub fn load_terms(
    archive: &mut ZipArchive<File>,
) -> Result<Vec<TermEntry>, Box<dyn std::error::Error>> {
    let terms = match archive.by_name("term_bank_1.json") {
        Ok(t) => t, // terms pointer so we can access term_bank_1.json contents
        Err(e) => return Err(e.into()),
    };

    // here term_entry is not an inst struct. its a vec of Tuples. each tuple has types
    // explicitly declared and comes from the json. we dont use all these just the 4 we want for our struct
    let term_entry: Vec<(
        String,            // term: 食べる KEEP
        Option<String>, // reading: たべる KEEP. Option<> because this field can be empty. Kana words for example wont have a reading field
        Option<String>, // definition tags SKIP. Option<> because also can be empty i think? either way we discard it later so jsut leave as is
        String,         // rules SKIP
        i64,            //score KEEP
        serde_json::Value, // definitions KEEP. the hard one. we process later
        i64,            // sequence number SKIP
        String,         // term tags //SKIP
    )> = match serde_json::from_reader(terms) {
        Ok(t) => t, // deserialize the whole term bank. each term in the json is an array of 8 elements (above). so array element 0 of the array goes in the tuple as t.0. and all those seperate tuples corresponding to different terms fill the Vec. Vec[0] is a the first term which is housed as a tuple of 8 fields (above)
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

pub fn extract_definition(definition: &serde_json::Value) -> String {
    // type A json: just definition string
    // if it's a string assign to s
    if let Some(s) = definition[0].as_str() {
        return String::from(s); // as_str returns &str so we convert to String
    }

    // placeholder
    String::new()
}

pub fn walk_tree(node: &serde_json::Value) -> Vec<String> {
    match node {
        serde_json::Value::String(_) => Vec::new(), // random nested strings we dont care. return empty vec
        serde_json::Value::Array(arr) => {
            let nested: Vec<Vec<String>> = arr.iter().map(|element| walk_tree(element)).collect();
            let flattened: Vec<String> = nested.into_iter().flatten().collect();
            flattened
        }
        serde_json::Value::Object(obj) => {
            let data = obj.get("data");
            if let Some(d) = data {
                let name = d.get("name");
                if let Some(n) = name {
                    if n.as_str() == Some("語釈") {
                        if let Some(content) = obj.get("content") {
                            if let Some(s) = content.as_str() {
                                return vec![s.to_owned()];
                            }
                        }
                    }
                }
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}
