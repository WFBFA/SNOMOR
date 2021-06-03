# OSM Extractor
_extract road graph from OSM snip_

The extractor accepts both OSM XML and OSM PBF formats.
While it can operate on raw extract, it is preferable (for faster processing) to pre-filter data to only include ways tagged "highway" (and used nodes).
For example, with [Osmosis](https://wiki.openstreetmap.org/wiki/Osmosis):
```sh
osmosis --read-xml "$1" --tf accept-ways highway=* --used-node --write-xml file="$1.roads.osm"
```
(or PBF)
```sh
osmosis --read-pbf-fast workers=8 "$1" --tf accept-ways highway=* --used-node --write-xml file="$1.roads.osm"
```
As an added bonus you can use Osmosis for to apply a poly filter with `--bp file="bounds.poly" completeWays=yes`.

The extractor runs on Node.
Run
```sh
node extract.js
```
to get started.

As per the [specification](https://github.com/WFBFA/Spec), the output is a [Road Graph](https://github.com/WFBFA/Spec/blob/main/1.road-graph.schema.json).

## Example usage
1. download Montreal area from [Overpass](https://overpass-api.de/api/map?bbox=-74.1660,45.2536,-73.2060,45.8652) and save as `montreal.osm`
2. (optional) compute Montreal area as `.poly` on https://polygons.openstreetmap.fr/index.py using relation [1571328](https://www.openstreetmap.org/relation/1571328) and save as [`montreal.poly`](https://polygons.openstreetmap.fr/get_poly.py?id=1571328&params=0).
2. (optional) Run it through osmosis `osmosis --read-xml "montreal.osm" --bp file="montreal.poly" completeWays=yes --tf accept-ways highway=* --used-node --write-xml file="montreal.roads.osm"`
3. run the extractor script `node extract.js montreal.roads.osm montreal.roads.json --nodes`
4. you now have the montreal road network graph in `montreal.roads.json`!
5. run geojsoniifier script `node extract.js montreal.roads.json montreal.roads.geojson` and visualize the produced GeoJSON on https://geojson.io/!

## Ambiguity resolution

OSM itself may be perfect, but the data there can be no better than wherever it comes from - contributors, aka people. That means that we have to deal with cases of incorrect, incomplete, and/or conflicting data... granted only occasionally (we're talking a few dozens road segments for Montreal agglomeration totalling many hundred thousands).

While there's nothing we can do about incorrect and/or incomplete data, it doesn't actually cause any problems. What is likely to cause serious problems though is amgigously conflicting data (ex: same section of a road specified twice, in different ways, once direction restricted and once bidirectional).

To deal with all of ambiguities, the data is sanitized, diluted, and resolved at the earliest stages of processing.
1. Sanitization - only `highway`s that are _clearly_ for wheeled vehicles are considered, and only if their parameters of interest are valid.
2. Dilution - instead of considering each way as an edge in the graph, _every intermediate line segment_ of the way is considered instead (with same tags as the way). The resulting edges are keyed on the 2 end nodes.
3. Resolution - when the graph already contains an edge with the same key, the tags are recombined and only 1 edge remains - the resulting road is bidirectional if either originator is, same for the sidewalks on both sides (length needs not be resolved as it is computed, later).

## Simplification

A complete road graph may well be complete, but it is *complete*ly unnecessary.

The simplification process is optional, enabled by default, reduces the complexity of the graph _without loss of precision_. In theory the process is trivial - merge every 2 or more consecutive edges (with same directionalit/sidewalks) without a 3rd road connecting in-between until there's none left. In practice, a discriminator property is added to differentiate between parallel roads.

Now, this section wouldn't exist, and the process would not be optional, would it have no caveats. Caveats:
- When the paths are produced, they pass over the same (simplified) graph, meaning they may not correspond 1:1 to physically existing/named roads.
- Navigation over a simplified graph (and therefor paths over it) is more complex (simplified edges need to be back-resolved into actual road segments, accounting for all corner cases - parallel edges, loops, _parallel looping simple edges_)

If the time allows, the WFBFA team may end up producing a path to GPX nav API, but for now that is not part of planned tasks.
