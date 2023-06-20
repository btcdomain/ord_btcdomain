use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindContent {
  #[clap(help = "Find inscribe by number.")]
  number: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub content: Vec<u8>,
  pub inscribe_num: i64,
  pub inscribe_id: String,
  pub timestamp: u32
}

impl FindContent {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;
    
    let output = index
      .get_inscription_id_by_inscription_number(self.number)
      .unwrap();
    let inscription_id = output.unwrap();
    let entry = index.get_inscription_entry(inscription_id).unwrap();

    let content = index.get_inscription_by_id(inscription_id).unwrap();
    if content.is_some() {
      print_json(Output {
        content: content.unwrap().into_body().unwrap(),
        inscribe_num: self.number,
        inscribe_id: inscription_id.to_string(),
        timestamp: entry.unwrap().timestamp
      })
      .unwrap();
      Ok(())
    } else {
      Err(anyhow!("query inscribe by number failed"))
    }
  }
}
