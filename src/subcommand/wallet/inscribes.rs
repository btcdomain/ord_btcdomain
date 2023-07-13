use {
  super::*,
  crate::wallet::Wallet,
  bitcoin::{
    blockdata::{opcodes, script},
    policy::MAX_STANDARD_TX_WEIGHT,
    schnorr::{UntweakedKeyPair},
    secp256k1::{
      self, constants::SCHNORR_SIGNATURE_SIZE, rand, schnorr::Signature, Secp256k1, XOnlyPublicKey,
    },
    util::sighash::{Prevouts, SighashCache},
    util::taproot::{ControlBlock, LeafVersion, TapLeafHash, TaprootBuilder},
    PackedLockTime, SchnorrSighashType, Witness,
  },
  std::collections::BTreeSet,
  log::info
};

#[derive(Serialize)]
struct Output {
  commit: Txid,
  inscription: Vec<InscriptionId>,
  reveal: Vec<Txid>,
  fees: u64,
}

#[derive(Debug, Parser)]
pub(crate) struct Inscribes {
  #[clap(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  pub(crate) fee_rate: FeeRate,
  #[clap(
    long,
    help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
  )]
  pub(crate) commit_fee_rate: Option<FeeRate>,
  #[clap(help = "Inscribes sat with contents of <FILE>")]
  pub(crate) file: PathBuf,
  #[clap(long, help = "Do not back up recovery key.")]
  pub(crate) no_backup: bool,
  #[clap(
    long,
    help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
  )]
  pub(crate) no_limit: bool,
  #[clap(long, help = "Don't sign or broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[clap(long, help = "Whether to use un-safe utxo.")]
  pub(crate) mint_size: u64,
  #[clap(long, help = "Send inscription to <DESTINATION>.")]
  pub(crate) destination: Option<Address>,
}

impl Inscribes {
  pub(crate) fn run(self, options: Options) -> Result {
    let inscription = Inscription::from_file(options.chain(), &self.file)?;

    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let mut utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let inscriptions = index.get_inscriptions(None)?;

    let reveal_tx_destination = match self.destination {
      Some(address) => {
        options
          .chain()
          .check_address_is_valid_for_network(&address)?;
        address
      }
      None => get_change_address(&client)?,
    };

    let commit_tx_change = get_change_address(&client)?;

    //过滤utxo
    let inscribesd_utxos = inscriptions
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();
    utxos  = utxos.iter()
    .filter(|(outpoint, _)| !inscribesd_utxos.contains(outpoint))
    .map(|(outpoint, amount)| (*outpoint, *amount))
    .collect();
    info!("inscribes utxos:{:?}",utxos);
    if utxos.is_empty() {
      bail!(
        "wallet contains no cardinal utxos"
      );
    }
    let (unsigned_commit_tx, reveal_tx_vec) =
      Inscribes::create_inscription_transactions(
        inscription,
        options.chain().network(),
        utxos.clone(),
        reveal_tx_destination,
        self.mint_size,
        self.fee_rate,
        self.no_limit,
        commit_tx_change,
      )?;
    let mut fees =Self::calculate_fee(&unsigned_commit_tx, &utxos);
    for reveal_tx in &reveal_tx_vec {
      utxos.insert(
        reveal_tx.input[0].previous_output,
        Amount::from_sat(
          unsigned_commit_tx.output[reveal_tx.input[0].previous_output.vout as usize].value,
        ),
      );
      fees=fees+Self::calculate_fee(&reveal_tx, &utxos);
    }
    if self.dry_run {
      let mut tx_id_vec=Vec::new();
      let mut inscription_vec=Vec::new();
      for reveal_tx in &reveal_tx_vec {
        tx_id_vec.push(reveal_tx.txid());
        inscription_vec.push(reveal_tx.txid().into());
      }
      print_json(Output {
        commit: unsigned_commit_tx.txid(),
        reveal: tx_id_vec,
        inscription: inscription_vec,
        fees,
      })?;
    } else {
      let signed_raw_commit_tx = client
        .sign_raw_transaction_with_wallet(&unsigned_commit_tx, None, None)?
        .hex;

      let commit = client
      .send_raw_transaction(&signed_raw_commit_tx)
      .context("Failed to send commit transaction")?;
      let mut tx_id_vec=Vec::new();
      let mut inscription_vec=Vec::new();
      for reveal_tx in &reveal_tx_vec {
        let reveal = client
          .send_raw_transaction(reveal_tx)
          .context("Failed to send reveal transaction")?;
        tx_id_vec.push(reveal);
        inscription_vec.push(reveal.into());
      }
      
      print_json(Output {
        commit,
        reveal: tx_id_vec,
        inscription: inscription_vec,
        fees,
      })?;
    }
    Ok(())
  }

  fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, Amount>) -> u64 {
    info!("calculate_fee tx:{:?} ,utxos:{:?}",tx,utxos);
    tx.input
      .iter()
      .map(|txin| utxos.get(&txin.previous_output).unwrap().to_sat())
      .sum::<u64>()
      .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<u64>())
      .unwrap()
  }

  fn create_inscription_transactions(
    inscription: Inscription,
    network: Network,
    utxos: BTreeMap<OutPoint, Amount>,
    destination: Address,
    mint_size: u64,
    reveal_fee_rate: FeeRate,
    no_limit: bool,
    change_address: Address,
  ) -> Result<(Transaction, Vec<Transaction>)> {
    let secp256k1 = Secp256k1::new();
    let key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
    let reveal_script_fee = inscription.append_reveal_script(
      script::Builder::new()
        .push_slice(&public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG),
    );
    let taproot_spend_info = TaprootBuilder::new()
      .add_leaf(0, reveal_script_fee.clone())
      .expect("adding leaf should work")
      .finalize(&secp256k1, public_key)
      .expect("finalizing taproot builder should work");

    let control_block = taproot_spend_info
      .control_block(&(reveal_script_fee.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let (_, reveal_fee) = Self::build_reveal_transaction(
      &control_block,
      reveal_fee_rate,
      OutPoint::null(),
      TxOut {
        script_pubkey: destination.script_pubkey(),
        value: 0,
      },
      &reveal_script_fee,
    );

    //计算reveal_fee 计算拆分多少个utxo
    let sum_amount = &utxos
      .iter()
      .map(|(_address, amount)| *amount)
      .sum::<Amount>();

    let mint_amount = reveal_fee + TransactionBuilder::TARGET_POSTAGE;
    let mut outputs: Vec<(Address, Amount)> = Vec::new();
    for _i in 0..mint_size {
      let reveal_script = inscription.append_reveal_script(
        script::Builder::new()
          .push_slice(&public_key.serialize())
          .push_opcode(opcodes::all::OP_CHECKSIG),
      );
      let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, reveal_script.clone())
        .expect("adding leaf should work")
        .finalize(&secp256k1, public_key)
        .expect("finalizing taproot builder should work");
      let split_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);
      outputs.push((split_tx_address.clone(), mint_amount));
    }
    //减去gas费 剩余的打回 找零地址
    let mut fee_outputs=outputs.clone();
    fee_outputs.push((change_address.clone(),Amount::from_sat(sum_amount.to_sat() - mint_amount.to_sat()*mint_size)));
    //计算vsize的消息
    let transaction = Transaction {
      version: 1,
      lock_time: PackedLockTime::ZERO,
      input: utxos.clone()
        .iter()
        .map(|outpoint| TxIn {
          previous_output: *outpoint.0,
          script_sig: Script::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        })
        .collect(),
      output: fee_outputs
        .iter()
        .map(|(address, amount)| TxOut {
          value: amount.to_sat(),
          script_pubkey: address.script_pubkey(),
        })
        .collect(),
    };
    let fee=reveal_fee_rate.fee(transaction.vsize());
    let change_fee = Amount::from_sat(sum_amount.to_sat() - mint_amount.to_sat()*mint_size-fee.to_sat());
    if change_fee<Amount::ZERO {
      bail!(
        "wallet contains no cardinal utxos"
      );
    }
    outputs.push((change_address.clone(),change_fee));

    //真实的消息
    let transaction = Transaction {
      version: 1,
      lock_time: PackedLockTime::ZERO,
      input: utxos
        .iter()
        .map(|outpoint| TxIn {
          previous_output: *outpoint.0,
          script_sig: Script::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        })
        .collect(),
      output: outputs
        .iter()
        .map(|(address, amount)| TxOut {
          value: amount.to_sat(),
          script_pubkey: address.script_pubkey(),
        })
        .collect(),
    };

    let mut reveal_tx_vec = Vec::new();
    for ele in transaction.output.iter().enumerate() {
      //排除找零
      if ele.1.value != mint_amount.to_sat() ||change_address.script_pubkey() == ele.1.script_pubkey {
        continue;
      }
      let reveal_script = inscription.append_reveal_script(
        script::Builder::new()
          .push_slice(&public_key.serialize())
          .push_opcode(opcodes::all::OP_CHECKSIG),
      );
      let (mut reveal_tx, fee) = Self::build_reveal_transaction(
        &control_block,
        reveal_fee_rate,
        OutPoint {
          txid: transaction.txid(),
          vout: ele.0 as u32,
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: ele.1.value,
        },
        &reveal_script,
      );

      reveal_tx.output[0].value = reveal_tx.output[0]
        .value
        .checked_sub(fee.to_sat())
        .context("commit transaction output value insufficient to pay transaction fee")?;

      if reveal_tx.output[0].value < reveal_tx.output[0].script_pubkey.dust_value().to_sat() {
        bail!("commit transaction output would be dust");
      }

      let mut sighash_cache = SighashCache::new(&mut reveal_tx);

      let signature_hash = sighash_cache
        .taproot_script_spend_signature_hash(
          0,
          &Prevouts::All(&[ele.1]),
          TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript),
          SchnorrSighashType::Default,
        )
        .expect("signature hash should compute");

      let signature = secp256k1.sign_schnorr(
        &secp256k1::Message::from_slice(signature_hash.as_inner())
          .expect("should be cryptographically secure hash"),
        &key_pair,
      );
      let witness = sighash_cache
        .witness_mut(0)
        .expect("getting mutable witness reference should work");
      witness.push(signature.as_ref());
      witness.push(reveal_script);
      witness.push(&control_block.serialize());

      let reveal_weight = reveal_tx.weight();

      if !no_limit && reveal_weight > MAX_STANDARD_TX_WEIGHT.try_into().unwrap() {
        bail!(
              "reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {reveal_weight}"
            );
      }
      reveal_tx_vec.push(reveal_tx);
    }
    Ok((transaction, reveal_tx_vec))
  }

  fn build_reveal_transaction(
    control_block: &ControlBlock,
    fee_rate: FeeRate,
    input: OutPoint,
    output: TxOut,
    script: &Script,
  ) -> (Transaction, Amount) {
    let reveal_tx = Transaction {
      input: vec![TxIn {
        previous_output: input,
        script_sig: script::Builder::new().into_script(),
        witness: Witness::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      }],
      output: vec![output],
      lock_time: PackedLockTime::ZERO,
      version: 1,
    };

    let fee = {
      let mut reveal_tx = reveal_tx.clone();

      reveal_tx.input[0].witness.push(
        Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
          .unwrap()
          .as_ref(),
      );
      reveal_tx.input[0].witness.push(script);
      reveal_tx.input[0].witness.push(&control_block.serialize());

      fee_rate.fee(reveal_tx.vsize())
    };

    (reveal_tx, fee)
  }
}