mod http;
mod helper;

use std::{collections::HashMap, env, fs, path::Path};

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 3 {
    eprintln!("Pass required environment variables");
    return;
  }
  dotenv::dotenv().ok();
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
  // println!("\nTarget directory: {:?}\nFile path of the target file: {:?}", target_dir, target_filepath);

  let target_content = match fs::read_to_string(target_filepath) {
    Ok(val) => val.trim().to_string(),
    Err(_) => {
      // eprintln!("{:?}", err);
      String::new()
    }
  };

  // println!("Content of the target file:");
  let target_lines = target_content.lines().collect::<Vec<_>>();
  // println!("{:#?}\n", target_lines);

  // println!("Content of the files to modify:");
  let filepaths_to_modify = match helper::read_public_folder(target_dir, target_filepath) {
    Ok(val) => { val },
    Err(_) => { vec![] }
  };

  let prompt = match fs::read_to_string("src/prompt_format.txt") {
    Ok(val) => val,
    Err(err) => { eprintln!("{:?}", err); String::new()}
  };

  for path in filepaths_to_modify {
    let dir_name = match Path::new(&path).parent() {
      Some(val) => val.as_os_str().to_str().unwrap_or_else(|| "").split("/").collect::<Vec<&str>>().last().unwrap_or_else(|| &""),
      _ => ""
    };

    println!("\nProcessing {}", dir_name);

    let mut generated_content: Vec<String> = vec![];
    let content = match fs::read_to_string(&path) {
      Ok(val) => val.trim().to_string(),
      Err(_) => { String::new() }
    };
    let lns = content.lines().collect::<Vec<_>>();
    // println!("\nlns: {:#?}", lns);

    let mut required_pairs: Vec<String> = vec![];

    for i in 0..target_lines.len() {
      let tl = target_lines[i].trim().replace(",", "");
      if tl.trim() == "{" || tl.trim() == "}" { continue; }

      let pair = tl.split(":").collect::<Vec<_>>();
      let target_key = helper::retrieve_key(pair[0]).unwrap_or_else(|| "");
      let key_exists_in_lns = lns.iter().any(|ln| ln.contains(target_key));

      if !key_exists_in_lns {
        required_pairs.push(tl);
      }
    }

    let formatted_prompt = prompt.replace("REQUIRED_KEY_VALUE_PAIRS", required_pairs.join(",").as_str()).replace("LANGUAGE_CODE", dir_name);
    // println!("formatted prompt: {:?}", formatted_prompt);
    let result = http::post_call(formatted_prompt).await;
    let dictionary = match helper::parse_json_response_as_hashmap(result) {
      Ok(val) => val,
      Err(_) => HashMap::new()
    };
    // println!("HashMap: {:?}", dictionary);

    for i in 0..target_lines.len() {
      let tl = target_lines[i].replace(",", "");
      if tl.trim() == "{" || tl.trim() == "}" { continue; }

      let pair = tl.split(":").collect::<Vec<_>>();
      let target_key = helper::retrieve_key(pair[0]).unwrap_or_else(|| "");
      // println!("{:?}: {:?} {:?}", i, target_key, lns.contains(&tl));
      let key_exists_in_lns = lns.iter().any(|ln| ln.contains(target_key));
      if key_exists_in_lns {
        if let Some(existing_line) = lns.iter().find(|ln| ln.contains(target_key)) {
          generated_content.push(existing_line.replace(",", ""));
        }
      } else {
        let translated_value = dictionary.get(target_key).map(|s| s.as_str()).unwrap_or("");
        generated_content.push(format!("\"{}\": \"{}\"", target_key, translated_value));
      }
    }
    let indented_content = generated_content.iter().map(|ln| format!("{}{}", " ".repeat(indentation_number), ln.trim())).collect::<Vec<_>>().join(",\n");
    let finalized_content = format!("{{\n{}\n}}\n", indented_content);
    // println!("Finalized content: {:#?}", finalized_content);
    let _ = fs::write(&path, finalized_content);
    println!("Done");
  }
}
