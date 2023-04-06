use std::path::PathBuf;

pub fn resolves_path(mut basedir: PathBuf, u: String) -> String {
    if u.starts_with("http://") || u.starts_with("https://") {
        u
    } else {
        if u.starts_with("/") {
            format!("file://{}", u)
        } else {
            basedir.push(u);
            format!("file://{}", basedir.to_str().unwrap())
        }
    }
}
