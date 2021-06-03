//! # _make 'em fly & make 'em plow_
//!
//! Universal CLI for solving flight & plow problems, as well as converting spec'd data to/from GeoJSON.

use std::borrow::Cow;

use clap::{App, Arg, SubCommand, crate_version};
mod data;
mod graph;
mod meta;
mod plow;
mod gj;
pub use try_all::{TryAll, TryMapAll};
pub use noisy_float::prelude::{N64, n64, Float};

pub type NodeId = Cow<'static, str>;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
enum Wut {
	Paths(data::Paths),
	Drones(data::Drones),
	Vehicles(data::VehiclesConfiguration),
	Snow(data::SnowStatuses),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
enum SnuwDapg {
	Formal(data::SnowStatuses),
	Geo(geojson::FeatureCollection),
}

/// Merge snow samplings with following rules:
/// - between a sample without snow and a sample with some snow, sampling with snow wins
/// - depths of all samples for given road segment are averaged
fn merge_snow_statuses(snows: impl Iterator<Item = data::SnowStatusElement>) -> data::SnowStatuses {
	let mut keyed = indexmap::IndexMap::new();
	for s in snows {
		let entry = keyed.entry((s.p1, s.p2, s.discriminator)).or_insert(n64(0.0));
		if *entry <= n64(0.0) || s.depth <= n64(0.0) {
			*entry = std::cmp::max(*entry, s.depth);
		} else {
			*entry = (*entry + s.depth) / n64(2.0);
		}
	}
	keyed.into_iter().map(|((p1, p2, discriminator), depth)| data::SnowStatusElement { p1, p2, discriminator, depth }).collect()
}

fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));
	let matches = App::new("Flight Paths Compute")
							.version(crate_version!())
							.about("Make it fly!")
							.subcommand(SubCommand::with_name("fly")
								.about("Compute flight paths")
								.arg(Arg::with_name("road-graph")
										.takes_value(true)
										.required(true)
										.index(1)
										.help("Road Graph JSON"))
								.arg(Arg::with_name("drones")
										.takes_value(true)
										.required(true)
										.index(2)
										.help("Drones configuration JSON"))
								.arg(Arg::with_name("meta")
										.takes_value(true)
										.required(true)
										.index(3)
										.help("Meta parameters"))
								.arg(Arg::with_name("output")
										.takes_value(true)
										.required(true)
										.index(4)
										.help("Output JSON"))
							)
							.subcommand(SubCommand::with_name("snows")
								.about("Merge multiple snow status updates")
								.arg(Arg::with_name("road-graph")
										.takes_value(true)
										.required(true)
										.index(1)
										.help("Road Graph JSON"))
								.arg(Arg::with_name("output")
										.takes_value(true)
										.required(true)
										.index(2)
										.help("Merged snow status output JSON"))
								.arg(Arg::with_name("snows")
										.takes_value(true)
										.required(true)
										.multiple(true)
										.help("Let it snow let it snow let it go")))
							.subcommand(SubCommand::with_name("plow")
								.about("Plow dat snow!")
								.arg(Arg::with_name("road-graph")
										.takes_value(true)
										.required(true)
										.index(1)
										.help("Road Graph JSON"))
								.arg(Arg::with_name("snow")
										.takes_value(true)
										.required(true)
										.index(2)
										.help("Snow status"))
								.arg(Arg::with_name("vehicles")
										.takes_value(true)
										.required(true)
										.index(3)
										.help("Vehicles configuration"))
								.arg(Arg::with_name("meta")
										.takes_value(true)
										.required(true)
										.index(4)
										.help("Meta parameters"))
								.arg(Arg::with_name("output")
										.takes_value(true)
										.required(true)
										.index(5)
										.help("Output JSON"))
								.arg(Arg::with_name("snow-d")
										.short("d")
										.takes_value(true)
										.default_value("0")
										.validator(|s| s.parse::<f64>().map(|_| ()).map_err(|e| e.to_string()))
										.help("Default snow depth"))	
								.arg(Arg::with_name("sidewalks")
									.short("w")
									.takes_value(false)
									.help("Clean sidewalks")))
							.subcommand(SubCommand::with_name("geojson")
								.about("Convert anything into GeoJSONs")
								.arg(Arg::with_name("road-graph")
										.takes_value(true)
										.required(true)
										.index(1)
										.help("Road Graph JSON"))
								.arg(Arg::with_name("wut")
										.takes_value(true)
										.required(true)
										.index(2)
										.help("Produced thingy that you want to convert (currently supported: flight paths)"))
								.arg(Arg::with_name("prefix")
										.takes_value(true)
										.required(true)
										.index(3)
										.help(r#"GeoJSON files prefix - the generated files will be named alike "{prefix}.{...}.geojson""#))
							)
							.get_matches();
	log::info!("Loading...");
	if let Some(matches) = matches.subcommand_matches("fly") {
		log::trace!("tracing enabled");
		let drones: data::Drones = serde_json::from_reader(&std::fs::File::open(matches.value_of("drones").unwrap())?).expect("Drones config invalid JSON");
		let roads: data::RoadGraph = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Road graph invalid JSON");
		let params: meta::Parameters = serde_yaml::from_reader(&std::fs::File::open(matches.value_of("meta").unwrap())?).expect("Meta parameters invalid JSON");
		log::info!("Loaded configuration");
		let paths = plow::fly::solve(roads, drones, &params).unwrap();
		log::info!("Constructed paths");
		serde_json::to_writer(&std::fs::File::create(matches.value_of("output").unwrap())?, &paths).unwrap();
	} else if let Some(matches) = matches.subcommand_matches("snows") {
		let roads: data::RoadGraph = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Road graph invalid JSON");
		log::info!("Loaded configuration");
		let mut snu: Vec<SnuwDapg> = Vec::new();
		for f in matches.values_of("snows").unwrap() {
			snu.push(serde_json::from_reader(&std::fs::File::open(f)?).expect("Snow status invalid JSON"));
		}
		log::info!("Loaded ❄");
		serde_json::to_writer(&std::fs::File::create(matches.value_of("output").unwrap())?, &merge_snow_statuses(snu.into_iter().map(|s| match s {
			SnuwDapg::Formal(s) => s,
			SnuwDapg::Geo(feat) => gj::geofeatures_to_snow(&roads, feat),
		}).flatten())).unwrap();
	} else if let Some(matches) = matches.subcommand_matches("plow") {
		log::trace!("tracing enabled");
		let roads: data::RoadGraph = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Road graph config invalid JSON");
		let snow: data::SnowStatuses = serde_json::from_reader(&std::fs::File::open(matches.value_of("snow").unwrap())?).expect("Snow status config invalid JSON");
		let vehicles: data::VehiclesConfiguration = serde_json::from_reader(&std::fs::File::open(matches.value_of("vehicles").unwrap())?).expect("Meta parameters invalid JSON");
		let params: meta::Parameters = serde_yaml::from_reader(&std::fs::File::open(matches.value_of("meta").unwrap())?).expect("Meta parameters invalid JSON");
		log::info!("Loaded configuration");
		if matches.is_present("sidewalks") {
			let paths = plow::sidewalk::solve(roads, snow, matches.value_of("snow-d").map(|f| f.parse().unwrap()), vehicles, &params).unwrap();
			log::info!("Constructed paths");
			serde_json::to_writer(&std::fs::File::create(matches.value_of("output").unwrap())?, &paths).unwrap();
		} else {
			let paths = plow::road::solve(roads, snow, matches.value_of("snow-d").map(|f| f.parse().unwrap()), vehicles, &params).unwrap();
			log::info!("Constructed paths");
			serde_json::to_writer(&std::fs::File::create(matches.value_of("output").unwrap())?, &paths).unwrap();
		}
	} else if let Some(matches) = matches.subcommand_matches("geojson") {
		let roads: data::RoadGraph = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Road graph config invalid JSON");
		let pref = matches.value_of("prefix").unwrap();
		let wut = serde_json::from_reader(&std::fs::File::open(matches.value_of("wut").unwrap())?).expect("WUT invalid JSON");
		log::info!("Loaded configuration");
		match wut {
			Wut::Paths(paths) => {
				let g = gj::roads_to_nodes(roads.nodes);
				for (i, path) in (0..paths.len()).zip(paths.into_iter()) {
					serde_json::to_writer(&std::fs::File::create(format!("{}.{}.geojson", pref, i))?, &gj::path_to_geojson(&g, path)).unwrap();
				}
			}
			Wut::Drones(drones) => {
				serde_json::to_writer(&std::fs::File::create(format!("{}.geojson", pref))?, &gj::locations_to_geojson(&roads.nodes, drones)).unwrap();
			}
			Wut::Vehicles(vc) => {
				serde_json::to_writer(&std::fs::File::create(format!("{}.road.geojson", pref))?, &gj::locations_to_geojson(&roads.nodes, vc.road)).unwrap();
				serde_json::to_writer(&std::fs::File::create(format!("{}.sidewalk.geojson", pref))?, &gj::locations_to_geojson(&roads.nodes, vc.sidewalk)).unwrap();
			}
			Wut::Snow(snows) => {
				serde_json::to_writer(&std::fs::File::create(format!("{}.geojson", pref))?, &gj::snows_to_geofeatures(&roads, snows)).unwrap();
			}
		}
	}
	Ok(())
}
