use monojiten::parser::load_dict_index;
use std::path::Path;

fn main() {
    let path = Path::new(
        "/Users/coleshanks/Documents/new_dic_shoui/[Monolingual] 明鏡国語辞典 第二版 Improved ver.zip",
    );

    let dict_index = match load_dict_index(path) {
        Ok(d) => d,
        Err(e) => panic!("Couldn't load the index\n{e}"),
    };

    println!("{}\n{}", dict_index.title, dict_index.format);
}
