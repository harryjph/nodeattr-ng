mod genders;
mod util;

use std::fs;
use crate::genders::Genders;

fn main() {
    let genders_file = fs::read_to_string("genders").unwrap();
    let genders = Genders::try_from(genders_file.as_str()).unwrap();
    println!("{genders:#?}");
}
