mod http;
mod helper;
mod cli;

use std::{fs, path::Path, time::Duration};
use clap::Parser;
use tokio::time::{sleep};

#[tokio::main]
async fn main() {
  let parsed_cli = cli::Cli::parse();

  dotenv::dotenv().ok();
  let root_dir = parsed_cli.root_dir;
  let lan_code = parsed_cli.lan_code.unwrap_or_else(|| String::from("en"));
  let file_names = parsed_cli.file.unwrap_or_else(|| {
    let values = match fs::read_dir(format!("{}/{}", root_dir, lan_code)) {
      Ok(val) => val.filter_map(|res| res.ok()).filter(|v| v.path().is_file()).filter_map(|e| e.path().file_name()?.to_str().map(|v| v.to_string())).collect::<Vec<String>>(),
      Err(_) => Vec::new(),
    };
    values
  });
  let indentation_number = parsed_cli.indent.unwrap_or_else(|| 4);
  let sleep_time_in_second = parsed_cli.sleep.unwrap_or_else(|| 0);
  let prompt_filepath = "src/prompt_format.txt";
  // println!("{lan_code:?} {file_names:?} {root_dir:?} {indentation_number:?}");

  let root_dir_path = Path::new(&root_dir);
  if !root_dir_path.exists() {
    eprintln!("Path {} does not exist.", &root_dir);
    return;
  }
  if !root_dir_path.is_dir() {
    eprintln!("Path {} is not a directory", &root_dir);
    return;
  }

  loop {
    let mut failed_task_count: u8 = 0;

    for file_name in &file_names {
      println!("\nProcessing {:?}", file_name);

      let reference_filepath = format!("{}/{}/{}", root_dir, lan_code, file_name);
      let reference_content = match fs::read_to_string(&reference_filepath) {
        Ok(val) => val.trim().to_string(),
        Err(_) => {
          eprintln!("Could not read file");
          String::new()
        }
      };

      // println!("Content of the reference file:");
      let reference_lines = reference_content.lines().collect::<Vec<_>>();
      // println!("{:#?}\n", reference_lines);

      // println!("Content of the files to modify:");
      let filepaths_to_modify = match helper::read_public_folder(&root_dir, &reference_filepath) {
        Ok(val) => { val },
        Err(_) => { vec![] }
      };
      // println!("filepaths_to_modify {:#?}", filepaths_to_modify);

      let prompt = match fs::read_to_string(prompt_filepath) {
        Ok(val) => val,
        Err(err) => {
          eprintln!("{:?}", err);
          String::new()
        }
      };

      for path in filepaths_to_modify {
        let dir_name = match Path::new(&path).parent() {
          Some(val) => val.as_os_str().to_str().unwrap_or_else(|| "").split("/").collect::<Vec<&str>>().last().unwrap_or_else(|| &""),
          _ => ""
        };
        println!("on {:?}", dir_name);

        let mut generated_content: Vec<String> = vec![];
        let content = match fs::read_to_string(&path) {
          Ok(val) => val.trim().to_string(),
          Err(_) => { String::new() }
        };
        let lns = content.lines().collect::<Vec<_>>();
        // println!("\nlns: {:#?}", lns);

        let mut required_pairs: Vec<String> = vec![];

        for i in 0..reference_lines.len() {
          let tl = reference_lines[i].trim().replace(",", "");
          if tl.trim() == "{" || tl.trim() == "}" { continue; }

          let pair = tl.split(":").collect::<Vec<_>>();
          let target_key = helper::retrieve_key(pair[0]).unwrap_or_else(|| "");
          let key_exists_in_lns = lns.iter().any(|ln| ln.contains(target_key));

          if !key_exists_in_lns {
            required_pairs.push(tl);
          }
        }

        if required_pairs.is_empty() {
          println!("Already synced!");
          continue;
        }

        let formatted_prompt = prompt.replace("REQUIRED_KEY_VALUE_PAIRS", required_pairs.join(",").as_str()).replace("LANGUAGE_CODE", dir_name);
        // println!("formatted prompt: {:?}", formatted_prompt);

        // sleep here for preventing 503 error from gemini request
        if sleep_time_in_second != 0 {
          sleep(Duration::from_secs(sleep_time_in_second)).await;
        }

        let result = http::post_call(formatted_prompt).await;
        let dictionary = match helper::parse_json_response_as_hashmap(result) {
          Ok(val) => val,
          Err(_) => {
            eprintln!("Failed to translate");
            failed_task_count = failed_task_count + 1;
            continue;
          }
        };
        // println!("HashMap: {:?}", dictionary);

        for i in 0..reference_lines.len() {
          let tl = reference_lines[i].replace(",", "");
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
            // println!("translated_value: {:?}", translated_value);
            generated_content.push(format!("\"{}\": \"{}\"", target_key, translated_value));
          }
        }
        let indented_content = generated_content.iter().map(|ln| format!("{}{}", " ".repeat(indentation_number), ln.trim())).collect::<Vec<_>>().join(",\n");
        let finalized_content = format!("{{\n{}\n}}\n", indented_content);
        // println!("\nFinalized content: {:#?}", finalized_content);
        let _ = fs::write(&path, finalized_content);
        println!("Done");
      }
    }

    if failed_task_count == 0 { break; }
    println!("\nSome translations failed. Wanna resolve those?\nType yes/y to continue and no/n to exit");
    let answer = cli::ask_user();
    if answer == "n" { break; }
  }

  println!("\nAll translations completed!");
}

// to delete all contents in files except files in folder en
// find public -type f ! -path "public/en/*" -exec sh -c '> "$1"' _ {} \;
