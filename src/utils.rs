use std::{
    fs::File,
    io::{BufReader, Read},
};

pub fn read_file_into_slice(path: &str, slice: &mut [u8]) {
    match File::open(path) {
        Ok(f) => {
            let mut reader = BufReader::new(f);
            let mut buffer = Vec::new();
            match reader.read_to_end(&mut buffer) {
                Ok(_) => slice.copy_from_slice(buffer.as_slice()),
                Err(e) => panic!("Could not read {path}! Error: {e}"),
            }
        }
        Err(e) => panic!("Could not open {path}! Error: {e}"),
    };
}
