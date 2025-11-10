use std::{collections::HashMap, error::Error, fs, path::Path};

pub fn read_public_folder(dir_path: &String, target: &String) -> Result<Vec<String>, std::io::Error> {
  let lang_dirs = fs::read_dir(dir_path)?
    .filter_map(|res| res.ok())
    .map(|res| res.path())
    .filter(|p| p.is_dir())
    .collect::<Vec<_>>();

  let mut result: Vec<String> = vec![];
  for dir_path in lang_dirs {
    let cur_filepath = fs::read_dir(dir_path)?
      .filter_map(|res| res.ok())
      .map(|res| res.path())
      .filter(|p| p.is_file() && p.to_str().map_or(false, |path_str| path_str != target))
      .find(|p| p.file_name() == Path::new(target).file_name())
      .map(|p| p.to_string_lossy().into_owned())
      .unwrap_or_else(|| String::new());

    if cur_filepath == "" { continue };
    result.push(cur_filepath);
  }
  Ok(result)
}

pub fn retrieve_key(s: &str) -> Option<&str> {
  let indices = (s.find('"'), s.rfind('"'));
  let (fi, li) = match indices {
    (Some(f), Some(l)) => (f, l),
    _ => return None
  };
  Some(&s[fi + 1..li])
}

pub fn parse_json_response_as_hashmap(result: Result<String, reqwest::Error>) -> Result<HashMap<String, String>, Box<dyn Error>> {
    // reqwest::Error implements Error, so the ? operator works to convert it to Box<dyn Error>
    let json_string = result?;
    // serde_json::Error also implements Error, so the ? operator works
    let dictionary: HashMap<String, String> = serde_json::from_str(&json_string)?;
    Ok(dictionary)
}
