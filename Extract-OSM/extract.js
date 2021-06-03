"use strict";
import fs, { watch } from 'fs';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import osm from 'osm-read';
import haversine from 'haversine-distance'

function processWay(way, roads, nodes){
	const highway = way.tags.highway;
	switch(highway){
		case 'motorway':
		case 'motorway_link':
		case 'trunk':
		case 'trunk_link':
		case 'primary':
		case 'primary_link':
		case 'secondary':
		case 'secondary_link':
		case 'tertiary':
		case 'tertiary_link':
		case 'unclassified':
		case 'residential':
		case 'living_street':
			break;
		default:
			return;
	}
	for(let n of way.nodeRefs) nodes.set(n, {});
	for(let i = 0; i < way.nodeRefs.length-1; i++){
		const r = {
			p1: way.nodeRefs[i],
			p2: way.nodeRefs[i+1],
			directed: way.tags.oneway === 'yes',
			sidewalks: way.tags.sidewalk === 'both' ? [true, true] : way.tags.sidewalk === 'left' ? [true, false] : way.tags.sidewalk === 'right' ? [false, true] : [false, false],
		};
		const k = JSON.stringify({
			p1: way.nodeRefs[i],
			p2: way.nodeRefs[i+1],
		});
		const exr = roads.get(k);
		if(exr){
			exr.directed &&= r.directed;
			exr.sidewalks[0] ||= r.sidewalks[0];
			exr.sidewalks[1] ||= r.sidewalks[1];
		} else roads.set(k, r);
	}
}

function finalize(file, roads, nodes, simplify){
	console.log("Populating distances")
	for(let r of roads){
		const p1 = nodes.get(r.p1);
		const p2 = nodes.get(r.p2);
		if(!p1 || !p2) throw new Error("Incomplete data!");
		r.distance = haversine(p1.coordinates, p2.coordinates);
	}
	if(simplify){
		console.log("Simplifying roads");
		const adj = new Map();
		for(let r of roads){
			let p1 = adj.get(r.p1) ?? [];
			p1.push(r);
			adj.set(r.p1, p1);
			let p2 = adj.get(r.p2) ?? [];
			p2.push(r);
			adj.set(r.p2, p2);
		}
		while(true){
			let simpd = 0;
			for(let [n, rs] of adj) if(rs.length === 2){
				const r1 = rs[0];
				const r2 = rs[1];
				if(r1.directed !== r2.directed || r1.sidewalks[0] !== r2.sidewalks[0] && r1.sidewalks[1] !== r2.sidewalks[1]) continue;
				let r;
				if(r1.p2 === r2.p1) r = { p1: r1.p1, p2: r2.p2 };
				else if(r2.p2 === r1.p1) r = { p1: r2.p1, p2: r1.p2 };
				else if(r1.p1 === r2.p1 && !r1.directed) r = { p1: r1.p2, p2: r2.p2 };
				else if(r1.p2 === r2.p2 && !r1.directed) r = { p1: r1.p1, p2: r2.p1 };
				else continue;
				r.discriminator = n;
				r.directed = r1.directed;
				r.sidewalks = r1.sidewalks;
				r.distance = r1.distance + r2.distance;
				const reli = (rs) => {
					if(rs.includes(r1)) rs.splice(rs.indexOf(r1), 1);
					if(rs.includes(r2)) rs.splice(rs.indexOf(r2), 1);
					rs.push(r);
				}
				reli(adj.get(r.p1));
				reli(adj.get(r.p2));
				rs.splice(0, 2);
				simpd++;
			}
			if(simpd == 0) break;
			console.log(` Reduced ${simpd}`);
		}
		roads = [...new Set([...adj.values()].flat())];
	}
	const usefulNodes = (function(){
		console.log("Stripping intermediate nodes");
		const retain = [];
		const yum = (p) => {
			if(p){
				nodes.delete(p.id);
				retain.push(p);
			}
		};
		for(let r of roads){
			yum(nodes.get(r.p1));
			yum(nodes.get(r.p2));
			yum(nodes.get(r.discriminator));
		}
		return retain;
	})();
	console.log(usefulNodes ? `Exporting (${roads.length} roads and ${usefulNodes.length} nodes)` : `Exporting (${roads.length} roads)`);
	fs.writeFileSync(file, JSON.stringify({
		roads,
		nodes: usefulNodes,
	}));
}

const argv = yargs(hideBin(process.argv))
	.command('extract <input> <output>', 'extract and transform OSM data', (yargs) => {
		return yargs
			.positional('input', { description: "OSM input data file (XML or PBF)" }).string('input')
			.positional('output', { description: "Output JSON file" }).string('output')
			.boolean('simplify').describe('simplify', "Simplify road geometry").default('simplify', true);
	}, (args) => {
		const nodes = new Map();
		const roads = new Map();
		console.log("First pass");
		(args.input.endsWith(".osm") ? osm.parseXml : osm.parse)({
			filePath: args.input,
			way: (way) => processWay(way, roads, nodes),
			error: (message) => {
				console.error(message);
				process.exit(1);
			},
			endDocument: () => {
				console.log("Second pass");
				(args.input.endsWith(".osm") ? osm.parseXml : osm.parse)({
					filePath: args.input,
					node: (node) => {
						if(nodes.has(node.id)) nodes.set(node.id, {
							id: node.id,
							coordinates: [node.lon, node.lat],
						});
					},
					error: (message) => {
						throw new Error(message);
					},
					endDocument: () => {
						finalize(args.output, [...roads.values()], nodes, args.simplify);
					},
				});
			},
		});
	})
	.command('geojson <input> <output>', "create GeoJSON from extracted data", (yargs) => {
		return yargs
			.positional('input', { description: "Input JSON file produced with `extract ... --nodes`" }).string('input')
			.positional('output', { description: "Output GeoJSON file" }).string('output')
			.boolean('viad').describe('viad', "Construct lines via discriminators").default('viad', false);
	}, (args) => {
		const data = JSON.parse(fs.readFileSync(args.input));
		if(!data?.roads || !data?.nodes) throw new Error("Can only use complete extracted data");
		const nodes = new Map();
		for(let n of data.nodes) nodes.set(n.id, n);
		fs.writeFileSync(args.output, JSON.stringify({
			type: 'GeometryCollection',
			geometries: data.roads.map(r => ({
				type: 'LineString',
				coordinates: args.viad && r.discriminator ? [nodes.get(r.p1).coordinates, nodes.get(r.discriminator).coordinates, nodes.get(r.p2).coordinates] : [nodes.get(r.p1).coordinates, nodes.get(r.p2).coordinates],
			})),
		}));
	})
	.help()
	.alias('help', 'h')
	.demandCommand(1)
	.argv;
