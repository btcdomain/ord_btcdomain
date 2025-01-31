use super::*;

pub mod epochs;
pub mod find_by_number;
pub mod find_content;
pub mod find;
mod index;
pub mod info;
pub mod list;
pub mod parse;
mod preview;
mod server;
pub mod subsidy;
pub mod supply;
pub mod traits;
pub mod wallet;
pub mod find_addr;
pub mod find_by_id;
pub mod find_cur_num;

fn print_json(output: impl Serialize) -> Result {
  serde_json::to_writer_pretty(io::stdout(), &output)?;
  println!();
  Ok(())
}

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[clap(about = "List the first satoshis of each reward epoch")]
  Epochs,
  #[clap(about = "Run an explorer server populated with inscriptions")]
  Preview(preview::Preview),
  #[clap(about = "query inscribe by number")]
  FindNumber(find_by_number::FindNumber),
  #[clap(about = "query inscribe content")]
  FindContent(find_content::FindContent),
  #[clap(about = "Find a satoshi's current location")]
  Find(find::Find),
  #[clap(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[clap(about = "Display index statistics")]
  Info(info::Info),
  #[clap(about = "List the satoshis in an output")]
  List(list::List),
  #[clap(about = "Parse a satoshi from ordinal notation")]
  Parse(parse::Parse),
  #[clap(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[clap(about = "Run the explorer server")]
  Server(server::Server),
  #[clap(about = "Display Bitcoin supply information")]
  Supply,
  #[clap(about = "Display satoshi traits")]
  Traits(traits::Traits),
  #[clap(subcommand, about = "Wallet commands")]
  Wallet(wallet::Wallet),
  #[clap(about = "query inscribe by number")]
  FindAddr(find_addr::FindAddr),
  #[clap(about = "query inscribe by id")]
  FindById(find_by_id::FindById),
  #[clap(about = "query current number")]
  FindCurNum(find_cur_num::FindCurNum),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Epochs => epochs::run(),
      Self::Preview(preview) => preview.run(),
      Self::FindNumber(find_number) => find_number.run(options),
      Self::FindContent(find_content) => find_content.run(options),
      Self::Find(find) => find.run(options),
      Self::Index(index) => index.run(options),
      Self::Info(info) => info.run(options),
      Self::List(list) => list.run(options),
      Self::Parse(parse) => parse.run(),
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(options, index, handle)
      }
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
      Self::Wallet(wallet) => wallet.run(options),
      Self::FindAddr(find_addr) => find_addr.run(options),
      Self::FindById(find_by_id) => find_by_id.run(options),
      Self::FindCurNum(find_cur_num) => find_cur_num.run(options),
    }
  }
}
