# Specification
_this is the global specification for the WFBFA project_

The project is split into multiple steps / stages of data processing.
Each step is implemented in its own repository with its own choice of langauge(s), frameworks, etc.
For reasons of ease of intercompatibility, the data between each step is exchanged in the JSON format.
This repository contains entirety of JSON Schemas for the data exchanged at every point - inputs and outputs of every step.
Unless the output of a step is already GeoJSON compliant, each step provides means to (re)export the data in GeoJSON format, mainly for visualization purposes.
Coordiantes system used implicitly follows [GeoJSON spec](https://datatracker.ietf.org/doc/html/rfc7946#section-3.1.1) - that is a position is a `[lon, lat]` pair, and everything that follows.

## [1 - Extraction of road graph](https://github.com/WFBFA/Extract-OSM)

The extractor filters through OSM data, extracting and repackaging relevant bits - automotive road network with sidewalk information - into easily useable format.

Inputs:
- OSM extract of region of interest (OSM XML or PBF file, pre-filtered to the region of interest)

Outputs:
- [Road graph](1.road-graph.schema.json)

## [2 - Construction of flight paths](https://github.com/WFBFA/Flight-Paths)

Using the road graph and surveillance vehicle configuration, optimal flight paths for complete coverage are constructed. 

Inputs:
- [Road graph](1.road-graph.schema.json)
- [Drone configuration](2.drones.schema.json)

Outputs:
- [Flight paths](2.flight-paths.schema.json) (start and end points of each path correspond to the base location specified in the input configuration)

## 3 - Brrrrr (processing of aerial imagery)

_implementation of this step is not part of the WFBFA project, and thus it must be implemented by the end user or a subcontractor following the specification._

Flying vehicles do their mass ❄ surveillance thing, and the resulting imagery is processed to produce up to date ❄ status.

Inputs:
- [Road graph](1.road-graph.schema.json)
- [Flight paths](2.flight-paths.schema.json)

Outputs:
- [Snow status](3.snow-status.schema.json)

## 4 - Construction of plow paths

Using all of the data collected so far and vehicle configuration, construct bestest itineraries for road and sidewalk snow removal vehicles.

Inputs:
- [Road graph](1.road-graph.schema.json)
- Snow status(es) - a default snow depth + 0 or more [snow status](3.snow-status.schema.json) \[surveillance\] results that \[each\] update previously accumulated information
- [Vehicle configuration](4.vehicles.schema.json)

Outputs:
- [Road vehicle paths](4.road-paths.schema.json)
- [Sidewalk vehicle paths](4.sidewalk-paths.schema.json)

## 5 - Costs modelling

//TODO :(
