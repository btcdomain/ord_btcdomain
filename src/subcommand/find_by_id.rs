use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindById {
  #[clap(help = "Find inscribe by id.")]
  id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub content: Vec<u8>,
  pub content_type: String,
  pub inscribe_num: i64,
  pub output_address: String,
  pub input_address: String
}

impl FindById {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let inscription_id = self.id.parse::<InscriptionId>().unwrap();
    let entry = index.get_inscription_entry(inscription_id).unwrap();

    let satpoint = index
      .get_inscription_satpoint_by_id(inscription_id)
      .unwrap()
      .unwrap();

    let tx = index.get_transaction(satpoint.outpoint.txid).unwrap().unwrap();
    let output_address = get_address_from_tx(options.chain().network(),satpoint.outpoint, &index);
    let input_address = get_address_from_tx(options.chain().network(),tx.input[0].previous_output, &index);

    let content = index.get_inscription_by_id(inscription_id).unwrap();
    if content.is_some() {
      let content_value = content.unwrap();
      print_json(Output {
        content: content_value.clone().into_body().unwrap(),
        content_type: (&content_value.content_type().unwrap()).to_string(),
        inscribe_num: entry.unwrap().number,
        output_address,
        input_address
      })
      .unwrap();
      Ok(())
    } else {
      Err(anyhow!("query inscribe by id failed"))
    }
  }
}

fn get_address_from_tx(network:Network,outpoint: OutPoint, index: &Index) -> String {
  let output = index.get_transaction(outpoint.txid).unwrap();
  if output.is_some() {
    let out_address = output
      .unwrap()
      .output
      .into_iter()
      .nth(outpoint.vout.try_into().unwrap())
      .map(|out| {
        Address::from_script(&out.script_pubkey, network).unwrap().to_string()
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