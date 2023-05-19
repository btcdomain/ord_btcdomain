
use super::*;

#[derive(Debug, Parser)]
pub(crate) struct TestInsertData {
}


impl TestInsertData {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.test_insert_data()?;
    Ok(())
  }
}
