//! GeoJSON conversion functions
//!
//! Converts "proprietary" data (in JSON following the [Spec](https://github.com/WFBFA/Spec)) to GeoJSON, mainly for visualization.
//! In some cases the conversion is reverisble, in which case GeoJSON to data converters are provided.

use crate::*;
use data::*;
use geo::{GeometryCollection, intersects::Intersects};

use std::{collections::HashSet, convert::{TryFrom, TryInto}};
use geojson::*;
use indexmap::{IndexMap, indexmap};

pub type Nodes = IndexMap<NodeId, Node>;

pub fn roads_to_nodes(g: RoadGraphNodes) -> Nodes {
	g.nodes.into_iter().map(|n| (n.id.clone(), n)).collect()
}

pub fn locations_to_geojson(g: &RoadGraphNodes, l: Vec<data::Location>) -> FeatureCollection {
	FeatureCollection {
		features: l.into_iter().map(|l| Feature {
			geometry: Some((&g.dislocate(&l)).try_into().unwrap()),
			properties: None,
			bbox: None,
			foreign_members: None,
			id: None
		}).collect(),
		bbox: None,
		foreign_members: None,
	}
}

pub fn path_to_geojson(g: &Nodes, path: Vec<PathSegment>) -> Geometry {
	Geometry::new(Value::LineString(path.into_iter().flat_map(|PathSegment { node, .. }| g.get(&node).map(|node| vec![node.coordinates.0, node.coordinates.1])).collect()))
}

pub fn geofeatures_to_snow(g: &RoadGraph, feat: FeatureCollection) -> data::SnowStatuses {
	let mut snow = Vec::new();
	for f in feat.features {
		if let (Some(depth), Some(geometry)) = (f.property("snow").and_then(|j| j.as_f64()), f.geometry) {
			let geometry: geo::Geometry<f64> = geometry.value.try_into().unwrap();
			let isect: HashSet<_> = g.nodes.nodes.iter().filter(|n| geometry.intersects(&geo::Geometry::<f64>::from(*n))).map(|n| &n.id).collect();
			for e in g.roads.iter().filter(|e| isect.contains(&e.p1) || isect.contains(&e.p2)) {
				snow.push(SnowStatusElement {
					p1: e.p1.clone(),
					p2: e.p2.clone(),
					discriminator: e.discriminator.clone(),
					depth: n64(depth),
				});
			}
		}
	}
	snow
}

pub fn snows_to_geofeatures(g: &RoadGraph, snow: data::SnowStatuses) -> FeatureCollection {
	let coords: IndexMap<_, _> = g.nodes.nodes.iter().map(|n| (&n.id, n.coordinates)).collect();
	FeatureCollection {
		features: snow.into_iter().map(|s| Feature {
			geometry: Some(Geometry::new(Value::LineString(vec![s.p1, s.p2].into_iter().map(|p| coords.get(&p).unwrap()).map(|(lon, lat)| vec![*lon, *lat]).collect()))),
			properties: Some(indexmap!{ "snow".to_string() => serde_json::to_value(s.depth).unwrap() }.into_iter().collect()),
			bbox: None,
			foreign_members: None,
			id: None,
		}).collect(),
		bbox: None,
		foreign_members: None,
	}
}
