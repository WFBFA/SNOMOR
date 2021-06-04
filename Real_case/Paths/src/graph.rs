//! Data structures and algorithms to operate on mixed generic graphs.
//! 
//! Historically, this was built after the algorithms in `brr` proved themselves worthy,
//! but the implementation was not sufficiently generic to be re-used for road, and then sidewalk, plowing.

use std::{collections::{HashMap, HashSet}, hash::Hash};

use indexmap::IndexMap;
use priority_queue::PriorityQueue;

/// An edge of a graph
///
/// Type Parameters:
/// - `NId`: node id
pub trait Edge<NId: Clone + Copy + Hash + Eq> : Clone + Hash + PartialEq + Eq {
	/// First (starting) vertex
	fn p1(&self) -> NId;
	/// Second (ending) vertex
	fn p2(&self) -> NId;
	/// Whether the edge is directed, i.e. can only be traversed `p1`→`p2`
	fn directed(&self) -> bool;
	/// Whether the edge is cyclic, i.e. goes from a vertex to itself
	fn is_cyclic(&self) -> bool {
		self.p1() == self.p2()
	}
	/// Assuming `id` is one end of the edge, what is the other end
	fn other(&self, id: NId) -> NId {
		if id == self.p1() {
			self.p2()
		} else {
			self.p1()
		}
	}
	/// Whether one can traverse the edge and end up in the specified node
	///
	/// Type Parameters:
	/// - `DIRESPECT`: whether the directionality of the edge is respected
	fn is_incoming<const DIRESPECT: bool>(&self, id: NId) -> bool {
		id == self.p2() || (id == self.p1() && (!DIRESPECT || !self.directed()))
	}
	/// Whether one can traverse the edge starting from the specified node
	///
	/// Type Parameters:
	/// - `DIRESPECT`: whether the directionality of the edge is respected
	fn is_outgoing<const DIRESPECT: bool>(&self, id: NId) -> bool {
		id == self.p1() || (id == self.p2() && (!DIRESPECT || !self.directed()))
	}
}

/// A graph
///
/// Type Parameters:
/// - `NId`: (lightweight) node id type
/// - `N`: Node type (can contain arbitrary node information)
/// - `E`: Edge type
#[derive(Clone, Debug)]
pub struct Graph<NId, N, E> 
where 
	NId: Clone + Copy + Hash + Eq,
	E: Edge<NId>,
{
	nodes: HashMap<NId, N>,
	edges: IndexMap<NId, HashSet<E>>,
	/// An always empty set of edges (useful for [`get_edges`] on a non-existing node)
	_empty: HashSet<E>,
}

impl<NId, N, E> Default for Graph<NId, N, E>
where 
	NId: Clone + Copy + Hash + Eq,
	E: Edge<NId>,
{
	fn default() -> Self {
		Self {
			nodes: Default::default(),
			edges: Default::default(),
			_empty: Default::default(),
		}
	}
}

impl<NId, N, E> Graph<NId, N, E>
where 
	NId: Clone + Copy + Hash + Eq,
	E: Edge<NId>,
{
	/// Constructs new graph with `nodes` and `edges`
	pub fn new(nodes: HashMap<NId, N>, edges: IndexMap<NId, HashSet<E>>) -> Self {
		Self { nodes, edges, ..Default::default() }
	}
	/// Get node by id
	pub fn get_node(&self, n: NId) -> Option<&N> {
		self.nodes.get(&n)
	}
	/// Get all edges of a node
	pub fn get_edges(&self, n: NId) -> &HashSet<E> {
		self.edges.get(&n).unwrap_or(&self._empty)
	}
	/// Whether the given node has no edges
	pub fn is_orphan(&self, n: NId) -> bool {
		self.get_edges(n).is_empty()
	}
	/// Get all edges between 2 nodes
	pub fn get_edges_between(&self, n1: NId, n2: NId) -> Vec<&E> {
		self.edges.get(&n1).iter().flat_map(|es| es.iter()).filter(|e| e.other(n1) == n2).collect()
	}
	/// Get all nodes
	pub fn nodes(&self) -> impl Iterator<Item=(NId,&N)> {
		self.nodes.iter().map(|(id, n)| (*id, n))
	}
	/// Get all edges
	pub fn edges(&self) -> impl Iterator<Item=&E> {
		self.edges.iter().flat_map(|(n, es)| es.iter().filter(move |e| e.is_cyclic() || e.p1() == *n))
	}
	/// Number of nodes
	pub fn node_count(&self) -> usize {
		self.nodes.len()
	}
	/// Number of edges
	pub fn edge_count(&self) -> usize {
		self.edges().count()
	}
	/// Whether the graph is empty
	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}
	/// Whether the graph has no edges
	pub fn is_edge_empty(&self) -> bool {
		self.edges.values().all(HashSet::is_empty)
	}
	/// Adds (or replaces) a node
	pub fn add_node(&mut self, id: NId, n: N) -> Option<N> {
		self.nodes.insert(id, n)
	}
	/// Adds an edge
	pub fn add_edge(&mut self, e: E) -> bool {
		if self.nodes.contains_key(&e.p1()) && self.nodes.contains_key(&e.p2()) {
			if !e.is_cyclic() {
				self.edges.entry(e.p1()).or_default().insert(e.clone());
			}
			self.edges.entry(e.p2()).or_default().insert(e);
			true
		} else {
			false
		}
	}
	/// Removes an edge
	pub fn remove_edge(&mut self, e: &E) -> bool {
		if self.nodes.contains_key(&e.p1()) && self.nodes.contains_key(&e.p2()) {
			if !e.is_cyclic() {
				self.edges.entry(e.p1()).or_default().remove(e);
			}
			self.edges.entry(e.p2()).or_default().remove(e);
			true
		} else {
			false
		}
	}
	/// Retains only the nodes (and edges) matching the predicate
	pub fn retain_nodes(&mut self, f: impl Fn(NId) -> bool){
		self.nodes.retain(|n, _| f(*n));
		self.retain_nodes_edges(f);
	}
	/// Retains only the edges of nodes matching the predicate
	pub fn retain_nodes_edges(&mut self, f: impl Fn(NId) -> bool){
		self.edges.retain(|n, _| f(*n));
		for (u, es) in &mut self.edges {
			es.retain(|e| f(e.other(*u)));
		}
	}
	/// Find all edges going from one region to another
	///
	/// Arguments:
	/// - `DIRESPECT`: whether the directionality of edges is respected
	/// - `n1`: nodes of the first region
	/// - `n2`: nodes of the second region
	///
	/// Returns: nodes `n1` and `n2` in the 1st and 2nd regions resp and the edge from `n1` to `n2`
	pub fn get_edges_between_regions<const DIRESPECT: bool>(&self, n1: &HashSet<NId>, n2: &HashSet<NId>) -> Vec<(NId, NId, &E)> {
		let mut es = Vec::new();
		for n1 in n1.iter().cloned() {
			for n2 in n2.iter().cloned() {
				for e in self.get_edges_between(n1, n2) {
					if e.is_outgoing::<DIRESPECT>(n1) && e.is_incoming::<DIRESPECT>(n2) {
						es.push((n1, n2, e));
					}
				}
			}
		}
		es
	}
	/// Find shortest path between 2 points, edge-weighted by a function
	///
	/// Currently uses heap-optimized Dijkstra's shortest path algorithm.
	///
	/// Type Parameters:
	/// - `Weight`: weight of an edge
	/// - `DIRESPECT`: whether the directionality of edges is respected
	///
	/// Arguments:
	/// - `n1`: first node
	/// - `n2`: second node
	/// - `weight`: filtering weight function - returns the weight of the edge, iff it can be traversed
	///
	/// Returns: edges path from `n1` to `n2`, if such exists
	pub fn pathfind<Weight, FW, const DIRESPECT: bool>(&self, n1: NId, n2: NId, weight: FW) -> Option<Vec<&E>>
	where
		Weight: Clone + Copy + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		let mut dp: HashMap<NId, (Weight, Option<&E>)> = HashMap::new();
		dp.insert(n1.clone(), (Weight::default(), None));
		let mut q = PriorityQueue::new();
		q.push(n1.clone(), Weight::default());
		while let Some((u, _)) = q.pop() {
			if u == n2 {
				let mut path = Vec::new();
				let mut v = u;
				while let Some((_, Some(e))) = dp.get(&v) {
					v = e.other(v);
					path.push(e.clone());
				}
				path.reverse();
				return Some(path);
			}
			let d = dp.get(&u).unwrap().0;
			for e in self.get_edges(u) {
				if !DIRESPECT || !e.directed() || e.p1() == u {
					if let Some(ed) = weight(e){
						let v = e.other(u);
						let d = d + ed;
						if dp.get(&v).map_or(true, |(vd, _)| vd > &d) {
							dp.insert(v.clone(), (d, Some(e)));
							q.push(v.clone(), -d);
						}
					}
				}
			}
		}
		None
	}
	/// Find shortest path between 2 regions, edge-weighted by a function
	///
	/// Currently uses heap-optimized Dijkstra's shortest path algorithm.
	///
	/// Type Parameters:
	/// - `Weight`: weight of an edge
	/// - `DIRESPECT`: whether the directionality of edges is respected
	///
	/// Arguments:
	/// - `n1`: nodes of the first region
	/// - `n2`: nodes of the second region
	/// - `weight`: filtering weight function - returns the weight of the edge, iff it can be traversed
	///
	/// Returns: nodes `n1` and `n2` in the 1st and 2nd regions resp and the edges path from `n1` to `n2`, if such exists
	pub fn pathfind_regions<Weight, FW, const DIRESPECT: bool>(&self, n1: &HashSet<NId>, n2: &HashSet<NId>, weight: FW) -> Option<(NId, NId, Vec<&E>)>
	where
		Weight: Clone + Copy + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		if n1.is_empty() || n2.is_empty() {
			return None;
		}
		let mut dp: HashMap<NId, (Weight, Option<&E>)> = HashMap::new();
		let mut q = PriorityQueue::new();
		for n1 in n1 {
			dp.insert(n1.clone(), (Weight::default(), None));
			q.push(n1.clone(), Weight::default());
		}
		while let Some((u, _)) = q.pop() {
			if n2.contains(&u) {
				let mut path = Vec::new();
				let mut v = u;
				while let Some((_, Some(e))) = dp.get(&v) {
					v = e.other(v);
					path.push(e.clone());
				}
				path.reverse();
				return Some((v, u, path));
			}
			let d = dp.get(&u).unwrap().0;
			for e in self.get_edges(u) {
				if !DIRESPECT || !e.directed() || e.p1() == u {
					if let Some(ed) = weight(e){
						let v = e.other(u);
						let d = d + ed;
						if dp.get(&v).map_or(true, |(vd, _)| vd > &d) {
							dp.insert(v.clone(), (d, Some(e)));
							q.push(v.clone(), -d);
						}
					}
				}
			}
		}
		None
	}
	/// Detect all strongly connected components in the graph
	///
	/// Currently uses unrecursed Tarjan's SCC algorithm.
	///
	/// Arguments:
	/// - `DIRESPECT`: whether the directionality of edges is respected
	/// - `ORPHANS`: whether orphan nodes are included as SCCs
	pub fn strongly_connected_components<const DIRESPECT: bool, const ORPHANS: bool>(&self) -> Vec<HashSet<NId>>
	where NId: std::fmt::Display {
		use std::cmp::min;
		let mut sccs = Vec::new();
		let mut index = 0usize;
		let mut stack = Vec::new();
		let mut inf: HashMap<_, (bool, usize, usize)> = HashMap::new();
		let mut q = Vec::new();
		for u in self.nodes.keys().into_iter().cloned() {
			if self.is_orphan(u) && !ORPHANS {
				continue;
			}
			if !inf.contains_key(&u) {
				q.push((u, self.get_edges(u).iter().collect::<Vec<_>>(), false));
				// "strongconnect"
				'unrec: while let Some((u, es, jr)) = q.last_mut() {
					let u = *u;
					// first visit
					if !inf.contains_key(&u) {
						stack.push(u);
						inf.insert(u, (true, index, index));
						index = index + 1;
					}
					// look at successors
					while let Some(e) = es.last() {
						if e.is_outgoing::<DIRESPECT>(u) {
							let v = e.other(u);
							let iv = inf.get(&v).cloned();
							let (.., ull) = inf.get_mut(&u).unwrap();
							match iv {
								// v has not yet been visited
								None => {
									*jr = true;
									q.push((v, self.get_edges(v).iter().collect::<Vec<_>>(), false));
									continue 'unrec;
								},
								// v was just visited
								Some((.., vll)) if *jr => {
									*ull = min(*ull, vll);
									*jr = false;
								},
								// v is in current scc
								Some((true, vidx, ..)) => {
									*ull = min(*ull, vidx)
								},
								_ => {}
							}
						}
						es.pop();
					}
					// generate scc
					let (_, idx, ll) = inf.get(&u).cloned().unwrap();
					if idx == ll {
						let mut scc = HashSet::new();
						loop {
							let v = stack.pop().unwrap();
							inf.get_mut(&v).unwrap().0 = false;
							scc.insert(v);
							if v == u {
								break;
							}
						}
						sccs.push(scc);
					}
					q.pop();
				}
			}
		}
		sccs
	}
	/// Patches weak links between regions
	///
	/// _SCCs together stronk!_
	///
	/// Arguments:
	/// - `DIRESPECT`: whether the directionality of edges is respected (weakly linked SCCs can only exist in mixed graphs, calling this without respect is no-op)
	/// - `regions`: regions between which to patch weak links, assumed SCCs
	/// - `dedirect`: function that transforms a directed edge into an undirected one, preserving all other properties (the function is always and only fed directed edges)
	pub fn patch_sccs<FD, const DIRESPECT: bool>(&mut self, regions: &Vec<HashSet<NId>>, dedirect: FD)
	where
		FD: Fn(E) -> E,
	{
		if DIRESPECT {
			let mut redir = HashSet::new();
			for i in 0..regions.len() {
				for j in (i+1)..regions.len() {
					for (.., e) in self.get_edges_between_regions::<false>(&regions[i], &regions[j]) {
						if e.directed() && !redir.contains(e) {
							redir.insert(e.clone());
						}
					}
				}
			}
			for e in &redir {
				self.remove_edge(e);
			}
			for e in redir.into_iter().map(dedirect) {
				self.add_edge(e);
			}
		}
	}
	/// Converts a path consisting of successive edges to successively visited nodes (with associated edges).
	///
	/// Example:
	/// - for path starting at `a` of edges `[(a;b),(c;b);(c;d)]` - returns `[a,b,c,d]`, or `[(a,_),(b,(a;b)),(c,(c;b)),(d,(c;d))]` with associated edeges.
	/// - an empty path starting at `a` - returns `[(a,_)]`
	pub fn path_to_nodes<'a>(path: impl Iterator<Item = &'a E>, n: NId) -> Vec<(NId, Option<&'a E>)> { //TODO this can become a generator one day!
		let mut vs = vec![(n, None)];
		for e in path {
			vs.push((e.other(vs.last().unwrap().0), Some(e)));
		}
		vs
	}
}

/// Graph construction adapters, for when your ids don't copy
pub mod adapt {
	use super::*;
	/// A Node that has an id associated to it
	pub trait IdentifiableNode {
		/// Id type
		type Id: Clone + Hash + Eq;
		/// Id of the node
		fn id(&self) -> &Self::Id;
	}
	/// A graph construction (id) adapter, to construct a graph with additional id mapping.
	///
	/// For alogrithmic performance reasons, [`Graph`] requires that node ids are [`Copy`].
	/// However that is not always the case.
	/// [`GraphAdapter`] hence allows you to construct a graph, by providing a stored "your node id" ↔ "graph node id" mapping.
	///
	/// Type Parameters:
	/// - `NId`: (lightweight) node id, used by the [`Graph`]
	/// - `E`: edge type
	/// - `N`: node type, that identifies itself to (heavy) node id
	/// - `IdAcc`: intermediate accumulator type value useful for [`GraphAdapter::new`]
	pub struct GraphAdapter<NId, N, E, IdAcc, Gen>
	where
		NId: Clone + Copy + Hash + Eq,
		E: Edge<NId>,
		N: IdentifiableNode,
		Gen: Fn(&N::Id, IdAcc) -> (NId, IdAcc),
	{
		pub graph: Graph<NId, N, E>,
		fwd: HashMap<N::Id, NId>,
		last_id: IdAcc,
		next_id: Gen,
	}
	impl<NId, N, E, IdAcc, Gen> GraphAdapter<NId, N, E, IdAcc, Gen>
	where
		NId: Clone + Copy + Hash + Eq,
		E: Edge<NId>,
		N: IdentifiableNode,
		IdAcc: Default,
		Gen: Fn(&N::Id, IdAcc) -> (NId, IdAcc),
	{
		/// Construct a new adapter.
		///
		/// Arguments:
		/// - `gen`: lightweight id generator function - given heavy node id and intermediate accumulator value, provide lightweight id and the next accumulator value
		/// - `acc`: initial intermediate accumulator value
		pub fn new(acc: IdAcc, gen: Gen) -> Self {
			Self {
				graph: Default::default(),
				fwd: Default::default(),
				last_id: acc,
				next_id: gen,
			}
		}
		/// Map heavy id to light id
		pub fn id2nid(&self, n: &N::Id) -> Option<NId> {
			self.fwd.get(n).map(|nid| *nid)
		}
		/// Map light id to node
		pub fn nid2node(&self, nid: NId) -> Option<&N> {
			self.graph.get_node(nid)
		}
		/// Map light id to heavy id
		pub fn nid2id(&self, nid: NId) -> Option<&N::Id> {
			self.nid2node(nid).map(|n| n.id())
		}
		/// Add a node to the graph, with id mappings
		pub fn add_node(mut self, n: N) -> Self {
			let (nid, acc) = (self.next_id)(n.id(), self.last_id);
			self.last_id = acc;
			self.fwd.insert(n.id().clone(), nid);
			self.graph.add_node(nid, n);
			self
		}
		/// Add an edge
		pub fn add_edge(&mut self, e: E) -> &mut Self {
			self.graph.add_edge(e);
			self
		}
	}
}

/// Heuristic graph algorithms
pub mod heuristics {
	use super::*;
	
	/// Solve Positioned Windy Rural Postman
	///
	/// Arguments:
	/// - `DIRESPECT`: respect directionality of edges
	/// - `g`: eulirian graph
	/// - `sp`: starting node
	/// - `alloc`: set of edges that need to be visited
	/// - `weight`: filtering weight function
	///
	/// Returns: the path visiting all allocated edges on success, or the allocated edges that can't be reached otherwise
	pub fn solve_pwrp<'a, NId, N, E, Weight, FW, const DIRESPECT: bool>(g: &'a Graph<NId, N, E>, sp: NId, mut alloc: HashSet<&'a E>, weight: FW) -> Result<Vec<&'a E>, HashSet<&'a E>>
	where 
		NId: Clone + Copy + Hash + Eq,
		E: Edge<NId>,
		Weight: Clone + Copy + PartialEq + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		log::trace!("Solving PWRP, starting with {}", alloc.len());
		let mut sol: Vec<&E> = Vec::new();
		macro_rules! sol_inject {
			($inj:expr,$y:expr) => {
				log::trace!("of {}", $inj.len());
				for e in &$inj {
					alloc.remove(e);
				}
				log::trace!("remaining {}", alloc.len());
				sol.splice($y..$y, $inj);
			}
		}
		while !alloc.is_empty() {
			if let Some((u, y, e)) = Graph::<NId, N, E>::path_to_nodes(sol.iter().map(|e| *e), sp).into_iter().enumerate().find_map(|(i, (u, _))| if let Some(e) = g.get_edges(u).iter().find(|e| e.is_outgoing::<DIRESPECT>(u) && alloc.contains(e)) { Some((u, i, e)) } else { None }) {
				log::trace!("injecting a cycle");
				let v = e.other(u);
				if let Some(mut p) = g.pathfind::<_, _, DIRESPECT>(v, u, |e| weight(e)) {
					p.insert(0, e);
					sol_inject!(p, y);
				} else {
					panic!("it's a trap!");
				}
			} else {
				log::trace!("connecting to a distant isle");
				let mut vs: HashSet<_> = alloc.iter().flat_map(|e| if !DIRESPECT || !e.directed() { vec![e.p1(), e.p2()] } else { vec![e.p1()] }).collect();
				let us: IndexMap<_, _> = Graph::<NId, N, E>::path_to_nodes(sol.iter().map(|e| *e), sp).into_iter().enumerate().map(|(i, (u, _))| (u, i)).collect();
				if let Some((inj, y)) = loop {
					if let Some((u, v, mut p)) = g.pathfind_regions::<_, _, DIRESPECT>(&us.keys().cloned().collect(), &vs, |e| weight(e)) {
						if let Some((e, mut pb)) = g.get_edges(v).iter().find_map(|e| if e.is_outgoing::<DIRESPECT>(v) && alloc.contains(e) {
							g.pathfind::<_, _, DIRESPECT>(e.other(v), u, |e| weight(e)).map(|path| (e, path))
						} else { None }) {
							p.push(e);
							p.append(&mut pb);
							// log::trace!("connecting {} to {} to {} to {}", u, v, e.other(v), u);
							break Some((p, *us.get(&u).unwrap()));
						} else {
							log::trace!("Can go from u to v, but not back; discarding v");
							vs.remove(&v);
						}
					} else {
						break None;
					}
				} {
					sol_inject!(inj, y);
				} else {
					log::trace!("failed to reach");
					return Err(alloc);
				}
			}
		}
		log::trace!("solved visiting {} segments", sol.len());
		Ok(sol)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	impl Edge<u64> for (u64, u64) {
		fn p1(&self) -> u64 {
			self.0
		}
		fn p2(&self) -> u64 {
			self.1
		}
		fn directed(&self) -> bool {
			true
		}
	}
	impl<W: Hash + Eq + Clone> Edge<u64> for (u64, u64, W) {
		fn p1(&self) -> u64 {
			self.0
		}
		fn p2(&self) -> u64 {
			self.1
		}
		fn directed(&self) -> bool {
			true
		}
	}

	macro_rules! graph {
		($edges:expr) => {
			{
				let mut g: Graph<_, _, _> = Default::default();
				for e in $edges {
					g.add_node(e.p1(), ());
					g.add_node(e.p2(), ());
					g.add_edge(e);
				}
				g
			}
		};
	}

	macro_rules! assert_eq_unordered {
		($left:expr, $right:expr) => {
			match (&$left, &$right) {
				(left, right) => {
					if left.len() != right.len() {
						assert_eq!(left, right);
					} else {
						for i in left {
							if !right.contains(i) {
								assert_eq!(left, right);
							}
						}
					}
				}
			}
		};
	}

	#[test]
	fn test_sccs(){
		let g = graph!(vec![(0, 1)]);
		assert_eq_unordered!(g.strongly_connected_components::<true, false>(), vec![vec![0].into_iter().collect(), vec![1].into_iter().collect()]);
		assert_eq_unordered!(g.strongly_connected_components::<false, false>(), vec![vec![0, 1].into_iter().collect()]);
		let g = graph!(vec![(0, 1), (1, 2), (2, 0), (3, 1), (3, 2), (4, 5), (5, 4)]);
		assert_eq_unordered!(g.strongly_connected_components::<true, false>(), vec![vec![0, 1, 2].into_iter().collect(), vec![3].into_iter().collect(), vec![4, 5].into_iter().collect()]);
		assert_eq_unordered!(g.strongly_connected_components::<false, false>(), vec![vec![0, 1, 2, 3].into_iter().collect(), vec![4, 5].into_iter().collect()]);
	}
}
