
use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Revert {
  height: u64
}


impl Revert {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.revert_height(self.height)?;
    Ok(())
  }
}
