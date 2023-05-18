
use super::*;

#[derive(Debug, Parser)]
pub(crate) struct TestInsertData {
  height: u64
}


impl TestInsertData {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.test_insert_data(self.height)?;
    Ok(())
  }
}
