//! Crusty data types for the [Specification](https://github.com/WFBFA/Specs)

use std::convert::TryFrom;

use crate::*;

use serde::*;

pub trait Distance {
	type Measure;
	fn distance(&self, other: &Self) -> Self::Measure;
}

impl Distance for (f64, f64) {
	type Measure = f64;
	fn distance(&self, othr: &Self) -> Self::Measure {
		(self.0-othr.0)*(self.0-othr.0) + (self.1-othr.1)*(self.1-othr.1)
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RoadSegment {
	pub p1: NodeId,
	pub p2: NodeId,
	pub discriminator: Option<NodeId>,
	pub directed: bool,
	pub distance: N64,
	pub sidewalks: (bool, bool),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SidewalkSide {
	#[serde(rename="left")]
	Left,
	#[serde(rename="right")]
	Right,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Node {
	pub id: NodeId,
	pub coordinates: (f64, f64),
}
impl From<&Node> for geo::Geometry<f64> {
	fn from(n: &Node) -> Self {
		geo::Point::from(n.coordinates).into()
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoadGraph {
	pub roads: Vec<RoadSegment>,
	#[serde(flatten)]
	pub nodes: RoadGraphNodes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoadGraphNodes {
	pub nodes: Vec<Node>,
}

impl RoadGraphNodes {
	/// Locates a location to the node on the graph
	pub fn locate(&self, l: &Location) -> Option<NodeId> {
		match l {
			Location::Coordinates(lon, lat) => self.nodes.iter().min_by_key(|Node {coordinates, ..}| n64((*lon, *lat).distance(coordinates))).map(|n| n.id.clone()),
			Location::Node(n) => Some(n.clone()),
		}
	}
	/// Locates a location to geographical coordinates
	pub fn dislocate(&self, l: &Location) -> geo::Geometry<f64> {
		match l {
			Location::Coordinates(lon, lat) => geo::Point::from((*lon, *lat)).into(),
			Location::Node(nid) => self.nodes.iter().find(|n| &n.id == nid).unwrap().into(),
		}
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum Location {
	Coordinates(f64, f64),
	Node(NodeId),
}

pub type Drones = Vec<Location>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct VehiclesConfiguration {
	pub road: Vec<Location>,
	pub sidewalk: Vec<Location>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PathSegment {
	pub node: NodeId,
	pub discriminator: Option<NodeId>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SidewalkPathSegment {
	pub node: NodeId,
	pub discriminator: Option<NodeId>,
	pub side: Option<SidewalkSide>,
}

pub type Paths = Vec<Vec<PathSegment>>;
pub type SidewalkPaths = Vec<Vec<SidewalkPathSegment>>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SnowStatusElement {
	pub p1: NodeId,
	pub p2: NodeId,
	pub discriminator: Option<NodeId>,
	pub depth: N64,
}

pub type SnowStatuses = Vec<SnowStatusElement>;
