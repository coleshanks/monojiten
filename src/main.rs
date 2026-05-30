use monojiten::parser::{load_dict_index, open_archive};
use std::path::Path;

fn main() {
    let path = Path::new(
        "/Users/coleshanks/Documents/new_dic_shoui/[Monolingual] 明鏡国語辞典 第二版 Improved ver.zip",
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
}
