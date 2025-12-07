mod http;
mod helper;
mod cli;

use std::{fs, path::Path, time::Duration};
use clap::Parser;
use tokio::{io, time::sleep};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
  let parsed_cli = cli::Cli::parse();

  let mut root_dir = parsed_cli.root_dir;
  let root_path_buf = Path::new(&root_dir).to_path_buf();
  if let Ok(abs_path) = root_path_buf.canonicalize() {
    root_dir = abs_path.to_string_lossy().into_owned();
  }
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

      let reference_lines = reference_content.lines().collect::<Vec<_>>();
      let filepaths_to_modify = match helper::read_public_folder(&root_dir, &reference_filepath) {
        Ok(val) => { val },
        Err(_) => { vec![] }
      };

      for path in filepaths_to_modify {
        let dir_name = match Path::new(&path).parent() {
          Some(val) => val.as_os_str().to_str().unwrap_or_else(|| "").split("/").collect::<Vec<&str>>().last().unwrap_or_else(|| &""),
          _ => ""
        };

        eprint!("On {:?} ... ", dir_name);
        io::stdout().flush().await.expect("Failed to flush stdout");

        let mut generated_content: Vec<String> = vec![];
        let content = match fs::read_to_string(&path) {
          Ok(val) => val.trim().to_string(),
          Err(_) => { String::new() }
        };
        let lns = content.lines().collect::<Vec<_>>();

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
          print!("\r\x1B[K");
          println!("✅ Already synced on {:?}", dir_name);
          continue;
        }

        if sleep_time_in_second != 0 {
          sleep(Duration::from_secs(sleep_time_in_second)).await;
        }

        let result = http::post_method(required_pairs, dir_name.to_string()).await;
        let dictionary = match helper::parse_json_response_as_hashmap(result) {
          Ok(val) => val,
          Err(e) => {
            print!("\r\x1B[K");
            println!("❌ Failed to translate on {:?}: {:?}", dir_name, e);
            failed_task_count = failed_task_count + 1;
            continue;
          }
        };

        for i in 0..reference_lines.len() {
          let tl = reference_lines[i].replace(",", "");
          if tl.trim() == "{" || tl.trim() == "}" { continue; }

          let pair = tl.split(":").collect::<Vec<_>>();
          let target_key = helper::retrieve_key(pair[0]).unwrap_or_else(|| "");
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
        let _ = fs::write(&path, finalized_content);
        print!("\r\x1B[K");
        println!("✅ Done on {:?}", dir_name);
      }
    }

    if failed_task_count == 0 { break; }
    eprint!("\nSome translations failed. Wanna resolve those? [Y/n] ");
    let answer = cli::ask_user();
    if answer == "n" { break; }
  }

  println!("\nAll translations completed!");
}
