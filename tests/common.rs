use std::path::Path;
use std::fs;
use layer_sword::errors::raise;

pub fn testcase_initial<P: AsRef<Path> + Copy>(dir_vec: Vec<P>) {
    env_logger::builder().is_test(true).try_init().unwrap_or_else(|_| {});
    for dir_path in dir_vec.clone() {
        if dir_path.as_ref().exists() {
            fs::remove_dir_all(dir_path).unwrap_or_else(|_| {});
        }
        raise(fs::create_dir(dir_path));
    }
}

pub fn testcase_destroy<P: AsRef<Path> + Copy>(dir_vec: Vec<P>) {
    for dir_path in dir_vec.clone() {
        if dir_path.as_ref().exists() {
            fs::remove_dir_all(dir_path).unwrap_or_else(|_| {});
        }
    }
}