use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindCurNum {
  
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub inscribe_num: usize,
}

impl FindCurNum {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let query = index.get_inscriptions(None);
    print_json(Output {
      inscribe_num: query.unwrap().len(),
    })
    .unwrap();
    Ok(())
  }
}
