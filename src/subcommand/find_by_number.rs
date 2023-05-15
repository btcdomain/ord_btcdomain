use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindNumber {
  #[clap(help = "Find inscribe by number.")]
  number: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub content: Vec<u8>,
  pub inscribe_num: u64,
  pub inscribe_id: String,
  pub sat: u64,
  pub output_address: String,
  pub input_address: String
}

impl FindNumber {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let output = index
      .get_inscription_id_by_inscription_number(self.number)
      .unwrap();
    let inscription_id = output.unwrap();

    // println!("inscription_id: {:?}", inscription_id);
    // let entry = index.get_inscription_entry(inscription_id).unwrap();
    // // println!("entry: {:?}", entry);
    // let sat = if entry.is_some() {
    //   let sats = entry.unwrap().sat;
    //   // println!("sats: {:?}", sats);
    //   if sats.is_some() {
    //     sats.unwrap().0
    //   } else {
    //     0
    //   }
    // } else {
    //   0
    // };

    let satpoint = index
      .get_inscription_satpoint_by_id(inscription_id)
      .unwrap()
      .unwrap();

    // println!("satpoint: {:?}", satpoint);
    let tx = index.get_transaction(satpoint.outpoint.txid).unwrap().unwrap();
    let output_address = get_address_from_tx(satpoint.outpoint, &index);
    let input_address = get_address_from_tx(tx.input[0].previous_output, &index);

    let content = index.get_inscription_by_id(inscription_id).unwrap();
    if content.is_some() {
      print_json(Output {
        content: content.unwrap().into_body().unwrap(),
        inscribe_num: self.number,
        inscribe_id: inscription_id.to_string(),
        sat: 0,
        output_address,
        input_address
      })
      .unwrap();
      Ok(())
    } else {
      Err(anyhow!("query inscribe by number failed"))
    }
  }
}
fn get_address_from_tx(outpoint: OutPoint, index: &Index) -> String {
  let output = index.get_transaction(outpoint.txid).unwrap();
  if output.is_some() {
    let out_address = output
      .unwrap()
      .output
      .into_iter()
      .nth(outpoint.vout.try_into().unwrap())
      .map(|out| {
        Address::from_script(&out.script_pubkey, Network::Bitcoin).unwrap().to_string()
      });
    if out_address.is_some() {
      out_address.unwrap()
    }else {
      String::new()
    }
  }else {
    String::new()
  }
}