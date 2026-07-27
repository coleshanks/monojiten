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
    let term_banks: Vec<String> = archive
        .file_names()
        .filter(|bank| bank.starts_with("term_bank_") && bank.ends_with(".json"))
        .map(|bank| bank.to_string())
        .collect();

    let mut all_banks: Vec<TermEntry> = Vec::new();

    for bank in term_banks {
        let terms = match archive.by_name(&bank) {
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
            serde_json::Value, // definition KEEP. the hard one. we process later
            i64,            // sequence number SKIP
            String,         // term tags //SKIP
        )> = match serde_json::from_reader(terms) {
            Ok(t) => t, // deserialize the whole term bank. each term in the json is an array of 8 elements (above). so array element 0 of the array goes in the tuple as t.0. and all those seperate tuples corresponding to different terms fill the Vec. Vec[0] is a the first term which is housed as a tuple of 8 fields (above)
            Err(e) => return Err(e.into()),
        };

        // return a Vec of TermEntry structs
        let bank_terms: Vec<TermEntry> = term_entry
            .into_iter() // iterator to let us walk the vec tuple by tuple and use map below to match tuple fields to our struct
            .map(|t| TermEntry {
                // closure here can be though of as a fn. this fn takes a tuple from the iterator above. and returns a TermEntry struct after we map the fields we want. the closure lets us do this inline instead of making a seperate fn and calling it here
                term: t.0,       // term: 食べる
                reading: t.1,    // reading: たべる
                score: t.4,      // score
                definition: t.5, // definition
            })
            .collect(); // the step above passes us a struct. collect pushes it into a new empty Vec. once all structs are pushed collect will return this Vec
        all_banks.extend(bank_terms);
    }
    Ok(all_banks)
}

// takes definition field from our struct, which is serde_json data and returns a plain String of the actual definition
// dics have Types A, B, and C for how definitions are stored in the json array
pub fn extract_definition(definition: &serde_json::Value) -> String {
    // Type A: just definition string. so TermEntry.definition[0] contains a string. and thats what we want thats the whole definition. so we simply return that string
    if let Some(s) = definition[0].as_str() {
        return String::from(s); // as_str returns &str so we convert to String
    }

    let string_pieces = walk_tree(definition);
    string_pieces.concat()
}

// walks the json exhaustively and returns a vec of strings. our definition is somewhere in this vec
pub fn walk_tree(node: &serde_json::Value) -> Vec<String> {
    match node {
        // we encountered a plain String. this is like a leaf in the json array structure. we cant progress any further in this branch
        serde_json::Value::String(s) => {
            vec![s.clone()] // s is a refernce to the String we are matching on. s is &String. Our return type is Vec<String>. So we clone s to get ownership
        }
        // we encountered an array. array is like a branch of the json and nested within it can be any of the six serde_json::Value enum variants: null, bool, number, string, array, object
        serde_json::Value::Array(a) => {
            let mut results = Vec::new(); // initialize a new empty vec
            // for child in a { results.extend(walk_tree(child)) } does the same as the below for loop
            for child in a {
                let child_strings = walk_tree(child); //recursively call walk_tree on each child
                for s in child_strings {
                    results.push(s); // push the strings we find into results
                }
            }
            results
        }
        // we encountered an object. objects are Map<String, Value> in the json. for example "tag": "span" which like tells yomitan how to render. span is from html. can also be nested like "content": { "tag": "img", ... } which is why we recurse on obj matches
        serde_json::Value::Object(o) => {
            if let Some(tag) = o.get("tag") {
                // we want to throw out img specifically because it never leads to definitions. other tag value matches like "span" "div" "ruby" "a" the walker will traverse and look for strings. "img" often has no content or is img content which we dont want. so we isolate it. stop. and return an empty vec
                if tag.as_str() == Some("img") {
                    return vec![];
                }
            }
            match o.get("content") {
                Some(c) => walk_tree(c), // we've found a content key. this contains the definition we want. we follow it recursively down and extract the strings
                None => vec![],
            }
        }
        // to cover null bool and number enum variants that we dont care about
        _ => vec![],
    }
}
