//! Meta parameters for the ❄ plow problem annealing solver

use crate::*;
use serde::*;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Recycle {
	/// do not move cycles
	No,
	/// move cycles between adjacent tours from expensive to cheap tour
	ExpensiveToCheap,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Clearing {
	/// the vehicle clears only the allocated edges
	OnlyAllocated,
	/// the vehicle clears all edges
	All,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Reorder {
	/// don't reorder
	No,
	/// swap 2 at random
	Swap2Random,
	/// generate new random order
	RandomReorder,
	/// swap most and least used
	Swap2MostLeast,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Realloc {
	/// don't
	No,
	/// swap 2 random links
	Swap2Random,
	/// move a link from vehicle that does most to one that does least
	MostToLeast,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Annealing {
	pub main_iterations: u64, //MI
	pub ft_iterations: u64, //II
	pub starting_temperature: f64, //ST
	pub cooling_factor: f64, //RC
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Parameters {
	pub recycle: Recycle, //IV
	pub clearing: Clearing, //MD
	pub reorder: Reorder, //ChV
	pub realloc: Realloc, //MV
	pub annealing: Annealing,
	pub slowdown: N64,
	pub weight_total: N64,
	pub weight_max: N64,
}
