use bitcoin::hashes::Hash;
use bitcoin::{Address, PublicKey};
use clap;

use cmd;
use hal;

pub fn subcommand<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand_group("address", "work with addresses")
		.subcommand(cmd_create())
		.subcommand(cmd_inspect())
}

pub fn execute<'a>(matches: &clap::ArgMatches<'a>) {
	match matches.subcommand() {
		("create", Some(ref m)) => exec_create(&m),
		("inspect", Some(ref m)) => exec_inspect(&m),
		(_, _) => unreachable!("clap prints help"),
	};
}

fn cmd_create<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand("create", "create addresses").args(&cmd::opts_networks()).args(&[
		cmd::opt_yaml(),
		cmd::opt("pubkey", "a public key in hex").takes_value(true).required(false),
		cmd::opt("script", "a script in hex").takes_value(true).required(false),
	])
}

fn exec_create<'a>(matches: &clap::ArgMatches<'a>) {
	let network = cmd::network(matches);

	let created = if let Some(pubkey_hex) = matches.value_of("pubkey") {
		let pubkey: PublicKey = pubkey_hex.parse().expect("invalid pubkey");
		hal::address::CreatedAddresses {
			p2pkh: Some(Address::p2pkh(&pubkey, network).to_string()),
			p2wpkh: Some(Address::p2wpkh(&pubkey, network).to_string()),
			p2shwpkh: Some(Address::p2shwpkh(&pubkey, network).to_string()),
			..Default::default()
		}
	} else if let Some(script_hex) = matches.value_of("script") {
		let script_bytes = hex::decode(script_hex).expect("invalid script hex");
		let script = script_bytes.into();

		hal::address::CreatedAddresses {
			p2sh: Some(Address::p2sh(&script, network).to_string()),
			p2wsh: Some(Address::p2wsh(&script, network).to_string()),
			p2shwsh: Some(Address::p2shwsh(&script, network).to_string()),
			..Default::default()
		}
	} else {
		panic!("Can't create addresses without a pubkey");
	};

	cmd::print_output(matches, &created)
}

fn cmd_inspect<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand("inspect", "inspect addresses")
		.args(&[cmd::opt_yaml(), cmd::arg("address", "the address").required(true)])
}

fn exec_inspect<'a>(matches: &clap::ArgMatches<'a>) {
	let address_str = matches.value_of("address").expect("no address provided");
	let address: Address = address_str.parse().expect("invalid address format");
	let script_pk = address.script_pubkey();

	let mut info = hal::address::AddressInfo {
		network: address.network,
		script_pub_key: hal::tx::OutputScriptInfo {
			hex: Some(script_pk.to_bytes().into()),
			asm: Some(script_pk.asm()),
			address: None,
			type_: None,
		},
		type_: None,
		pubkey_hash: None,
		script_hash: None,
		witness_program_version: None,
	};

	use bitcoin::util::address::Payload;
	match address.payload {
		Payload::PubkeyHash(pkh) => {
			info.type_ = Some("p2pkh".to_owned());
			info.pubkey_hash = Some(pkh.into_inner()[..].into());
		}
		Payload::ScriptHash(ref sh) => {
			info.type_ = Some("p2sh".to_owned());
			info.script_hash = Some(sh.into_inner()[..].into());
		}
		Payload::WitnessProgram {
			version,
			program,
		} => {
			let version = version.to_u8() as usize;
			info.witness_program_version = Some(version);

			if version == 0 {
				if program.len() == 20 {
					info.type_ = Some("p2wpkh".to_owned());
					info.pubkey_hash = Some(program.into());
				} else if program.len() == 32 {
					info.type_ = Some("p2wsh".to_owned());
					info.script_hash = Some(program.into());
				} else {
					info.type_ = Some("invalid-witness-program".to_owned());
				}
			} else {
				info.type_ = Some("unknown-witness-program-version".to_owned());
			}
		}
	}

	cmd::print_output(matches, &info)
}
