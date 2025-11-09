use std::{env, fs, io::Error, path::Path};

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 3 {
    eprintln!("Pass required environment variables");
    return;
  }
  let (target_dir, target_filepath) = (&args[1], &args[2]);
  let default_indentation_number = 4;
  let indentation_number = match args.get(3) {
    Some(arg) => {
      match arg.parse() {
        Ok(val) => val,
        Err(_) => default_indentation_number
      }
    },
    _ => default_indentation_number
  };
  println!("\nTarget directory: {:?}\nFile path of the target file: {:?}", target_dir, target_filepath);

  let target_content = match fs::read_to_string(target_filepath) {
    Ok(val) => val.trim().to_string(),
    Err(err) => {
      eprintln!("{:?}", err);
      String::new()
    }
  };

  println!("Content of the target file:");
  let target_lines = target_content.lines().collect::<Vec<_>>();
  // println!("{:#?}\n", target_lines);

  println!("Content of the files to modify:");
  let filepaths_to_modify = match read_public_folder(target_dir, target_filepath) {
    Ok(val) => { val },
    Err(_) => { vec![] }
  };

  for path in filepaths_to_modify {
    let mut generated_content: Vec<String> = vec![];
    let content = match fs::read_to_string(&path) {
      Ok(val) => val.trim().to_string(),
      Err(_) => { String::new() }
    };
    let lns = content.lines().collect::<Vec<_>>();
    println!("\nlns: {:#?}", lns);

    for i in 0..target_lines.len() {
      let tl = target_lines[i];
      if tl.trim() == "{" || tl.trim() == "}" {
        continue;
      }
      let pair = tl.split(":").collect::<Vec<_>>();
      let target_key = match retrieve_key(pair[0]) {
        Some(val) => val,
        _ => ""
      };
      println!("{:?}: {:?} {:?}", i, target_key, lns.contains(&tl));
      let key_exists_in_lns = lns.iter().any(|ln| ln.contains(target_key));
      if key_exists_in_lns {
        if let Some(existing_line) = lns.iter().find(|ln| ln.contains(target_key)) {
          generated_content.push(existing_line.replace(",", ""));
        }
      } else {
        generated_content.push(tl.replace(",", ""));
      }

      let indented_content = generated_content.iter().map(|ln| format!("{}{}", " ".repeat(indentation_number), ln.trim())).collect::<Vec<_>>().join(",\n");
      let finalized_content = format!("{{\n{}\n}}\n", indented_content);
      let _ = fs::write(&path, finalized_content);
      // println!("*** {:#?}", finalized_content);
    }

  }
}

fn read_public_folder(dir_path: &String, target: &String) -> Result<Vec<String>, Error> {
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

fn retrieve_key(s: &str) -> Option<&str> {
  let indices = (s.find('"'), s.rfind('"'));
  let (fi, li) = match indices {
    (Some(f), Some(l)) => (f, l),
    _ => return None
  };
  Some(&s[fi + 1..li])
}
