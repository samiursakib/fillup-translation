use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "filltra")]
#[command(version = "1.0.0")]
#[command(about = "This project aims to automate developer workflows who frequently comes across to generate translations for different languages from json object.")]
pub struct Cli {
  /// public root folder
  pub root_dir: String,

  /// language code to translate from (default = en)
  #[arg(long)]
  pub lan_code: Option<String>,

  /// list of files to process (default behavior is to process all files)
  #[arg(long)]
  pub file: Option<Vec<String>>,

  /// indentation of the translated json file (default = 4 spaces)
  #[arg(long)]
  pub indent: Option<usize>,

  /// sleep time between each request for translation (default = 0)
  #[arg(long)]
  pub sleep: Option<u64>,
}

pub fn ask_user() -> String {
  let mut answer = String::new();
  std::io::stdin().read_line(&mut answer).unwrap();

  match answer.trim().to_lowercase().as_str() {
    "y" | "yes" => String::from("y"),
    "n" | "no" => String::from("n"),
    _ => {
      eprint!("Invalid input. Please enter [Y/n] ");
      ask_user();
      String::from("n")
    }
  }
}
