use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindAddr {
  #[clap(help = "Find inscribe by id.")]
  id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub output_address: String,
  pub input_address: String
}

impl FindAddr {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let inscription_id = self.id.parse::<InscriptionId>().unwrap();

    let satpoint = index
      .get_inscription_satpoint_by_id(inscription_id)
      .unwrap()
      .unwrap();

    let tx = index.get_transaction(satpoint.outpoint.txid).unwrap().unwrap();
    let output_address = get_address_from_tx(satpoint.outpoint, &index);
    let input_address = get_address_from_tx(tx.input[0].previous_output, &index);

    print_json(Output {
      output_address,
      input_address
    })
    .unwrap();
    Ok(())
    
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