# Paths Compute

_make 'em fly & make 'em plow_

This is a universal paths compute & GeoJSON conversion multi-tool.

Using the road graph and surveillance vehicle configuration (in JSONs as per the schema), compute _somewhat_ optimal paths for perfect\* road coverage.

\* - unreachable road segments are unreachable and there's nothing i can do about it :P

The app is a Rust CLI - just run with `cargo bin`.

## Limitations

~~Current algorithm will not utilize all of the vehicles starting at the same graph node if there are more vehicles there than half the number of augmented edges at that node.~~ _Fixed in the next version_

The vehicles are allowed to follow the road graph and only. That means that if there are _logically_ disconnected portions, even if they are physically accessible, they will not be visited (and you will get an ~~warning~~ error).

Flight speed, traffic control, weather, fuel/🔋 mileage, and most other physical conditions are _not_ taken into account.

The lengths of paths of vehicles are balanced, to _some_ possible/reasonable extent.

## Drones

The `fly` command allows to compute drone paths for vehicles starting in specified locations.

#### Example usage
1. get ur road graph in `montreal.roads.json`
2. create a drone configuration in `drones.json`. for example
```json
[
	"596644787",
	"218198673",
	"4234468198"
]
```
3. run `cargo bin -- fly montreal.roads.json drones.json drones.paths.json`
4. the paths for the 3 drones are now in `drones.paths.json`
5. shalt thou wish to geojsonify it, run `cargo bin -- geojson montreal.roads.json drones.paths.json drones.path` and make use of the generated `drones.path.1.geojson`, `drones.path.2.geojson` and `drones.path.3.geojson` files.

## Snow Status Aggregation

The `snow` command allows aggregating multiple snow status informations into a single one. Additionally, multiple formats are supported:
- obviously, the WFBFA snow status JSON
- GeoJSON feature collection JSON - each feature specifying a `snow` (or `snow-depth`) numerical property is matched with road map and each intersecting road segment is assigned that depth

## Plowing

The `plow` command allows computing road cleaning vehicle paths starting in specified locations.

Meta parameters allow controlling the common behicle properties (slowdown for cleaning) as well as the parameters for annealing heuristic itself and score valuation weights.

Example meta parameters:
```yaml
recycle: ExpensiveToCheap
clearing: All
reorder: RandomReorder
realloc: No
slowdown: 2
weight_total: 1
weight_max: 10
annealing:
  main_iterations: 8
  ft_iterations: 2
  starting_temperature: 1000
  cooling_factor: 0.3
```

## GeoJSON

The `geojson` command allows converting different WFBFA JSONs into GeoJSON representation (where applicable, the output can be reversed back into original format.
This is mostly useful for doing fancy vizualizations/editing on [geojson.io](https://geojson.io) or [Large GeoJSON visualizer](https://e-gy.github.io/leaflet-geojson-large/).

Currently supported conversions:
- Snow
- Vehicles
- Paths
