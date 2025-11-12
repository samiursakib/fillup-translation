use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "MyApp")]
#[command(version = "1.0")]
#[command(about = "Does awesome things")]
pub struct Cli {
  pub root_dir: String,
  #[arg(long)]
  pub lan_code: Option<String>,
  #[arg(long)]
  pub file: Option<Vec<String>>,
  #[arg(long)]
  pub indent: Option<usize>,
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
      eprintln!("Invalid input. Please enter yes/y or no/n");
      ask_user();
      String::new()
    }
  }
}
