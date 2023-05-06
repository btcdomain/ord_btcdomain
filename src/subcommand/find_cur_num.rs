use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindCurNum {
  
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub inscribe_num: u64,
}

impl FindCurNum {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let query = index.get_latest_inscriptions_with_prev_and_next(1, None);
    if query.is_ok() {
      let inscribe_num = query.unwrap().2.unwrap() - 1;
      print_json(Output {
        inscribe_num: inscribe_num,
      })
      .unwrap();
      Ok(())
    }else {
      Err(anyhow!("query inscribe cur number failed"))
    }
    
  }
}
