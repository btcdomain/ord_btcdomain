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
  pub inscribe_num: u64,
  pub address: Vec<String>,
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

    let address = get_address_from_satpoint(satpoint.outpoint, &index);

    let content = index.get_inscription_by_id(inscription_id).unwrap();
    if content.is_some() {
      let content_value = content.unwrap();
      print_json(Output {
        content: content_value.clone().into_body().unwrap(),
        content_type: (&content_value.content_type().unwrap()).to_string(),
        inscribe_num: entry.unwrap().number,
        address: vec![address],
      })
      .unwrap();
      Ok(())
    } else {
      Err(anyhow!("query inscribe by id failed"))
    }
  }
}

fn get_address_from_satpoint(sat_point: OutPoint, index: &Index) -> String {
  let output = index.get_transaction(sat_point.txid).unwrap();
    let tx: Transaction = output.unwrap();
    let out_address = tx
      .output
      .iter()
      .nth(sat_point.vout.try_into().unwrap())
      .map(|tx_out| {
        let addr = Address::from_script(&tx_out.script_pubkey, Network::Bitcoin).unwrap().to_string();
        addr
    });

    if out_address.is_some() {
      out_address.unwrap()
    }else {
      String::new()
    }
}