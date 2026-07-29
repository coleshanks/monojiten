use monojiten::parser::{load_dict_index, load_terms, open_archive, extract_definition};
use monojiten::lookup::find_terms;
use std::path::Path;

fn main() {
    let path = Path::new(
        "/Users/coleshanks/Documents/Projects/cli/monojiten/dictionaries/[Monolingual] 明鏡国語辞典 第二版 Improved ver.zip",
        // "/Users/coleshanks/Documents/Projects/cli/monojiten/dictionaries/[Monolingual] 大辞林 第三版.zip",
    );

    let mut archive = match open_archive(path) {
        Ok(a) => a,
        Err(e) => panic!("Couldn't open the archive\n{e}"),
    };

    let dict_index = match load_dict_index(&mut archive) {
        Ok(d) => d,
        Err(e) => panic!("Couldn't load the index\n{e}"),
    };

    println!("{}\n{}", dict_index.title, dict_index.format);

    let terms = match load_terms(&mut archive) {
        Ok(t) => t,
        Err(e) => panic!("Couldn't load the terms\n{e}"),
    };

    println!("Let's read some stuff from term_bank_1.json!");

    // println!(
    //     "term: {}\n reading: {:?}\n score: {}\n definition: {:?}",
    //     terms[9].term, terms[9].reading, terms[9].score, terms[9].definition,
    // );

    // for i in terms {
    //     println!(
    //         "term: {}\n reading: {:?}\n score: {}\n definition: {}\n",
    //         i.term,
    //         i.reading,
    //         i.score,
    //         extract_definition(&i.definition)
    //     );
    // }

    for i in terms {
        println!(
            "term: {}\n definition: {}\n",
            i.term,
            extract_definition(&i.definition)
        );
    }
}
