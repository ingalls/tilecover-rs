extern crate geo;

use std::f64::consts::PI;
use std::f64::INFINITY as INF;
use std::cmp::Ordering;
use geo::*;

const D2R: f64 = PI / 180.0;
const R2D: f64 = 180.0 / PI;

#[derive(Debug, PartialEq)]
pub enum Error {
    GeomTypeNotSupported
}

pub fn tiles(geom: &Geometry<f64>, zoom: u8) -> Result<Vec<(i32, i32, u8)>, Error> {
    match geom {
        &geo::Geometry::Point(ref point) => {
            Ok(vec!(point_to_tile(point.lat(), point.lng(), zoom)))
        },
        &geo::Geometry::MultiPoint(ref points) => {
            let mut tiles: Vec<(i32, i32, u8)> = Vec::new();

            for point in points.clone() {
                let tile = point_to_tile(point.lat(), point.lng(), zoom);
                if !tiles.contains(&tile) {
                    tiles.push(tile)
                }
            }

            Ok(tiles)
        },
        &geo::Geometry::LineString(ref linestring) => {
            let mut tiles: Vec<(i32, i32, u8)> = Vec::new();

            line_cover(&mut tiles, linestring, zoom, None);

            tiles.sort();
            tiles.dedup();

            Ok(tiles)
        },
        &geo::Geometry::MultiLineString(ref linestrings) => {
            let mut tiles: Vec<(i32, i32, u8)> = Vec::new();

            for ref linestring in linestrings.0.iter() {
                line_cover(&mut tiles, linestring, zoom, None);
            }

            tiles.sort();
            tiles.dedup();

            Ok(tiles)
        },
        &geo::Geometry::Polygon(ref polygon) => {
            let mut tiles: Vec<(i32, i32, u8)> = Vec::new();

            poly_cover(&mut tiles, polygon, zoom);

            tiles.sort();
            tiles.dedup();

            Ok(tiles)
        },
        &geo::Geometry::MultiPolygon(ref polygons) => {
            let mut tiles: Vec<(i32, i32, u8)> = Vec::new();

            for ref polygon in polygons.0.iter() {
                poly_cover(&mut tiles, polygon, zoom);
            }

            tiles.sort();
            tiles.dedup();

            Ok(tiles)
        },
        _ => Err(Error::GeomTypeNotSupported)
    }
}

pub fn poly_cover(tiles: &mut Vec<(i32, i32, u8)>, polygon: &geo::Polygon<f64>, zoom: u8) {
    let mut intersections: Vec<(i32, i32)> = Vec::new();

    poly_cover_single(&mut intersections, tiles, &polygon.exterior, zoom);

    for interior in &polygon.interiors {
        poly_cover_single(&mut intersections, tiles, &interior, zoom);
    }

    // sort by y, then x
    intersections.sort_by(|a,b| {
        //Sort by y first
        if a.1 > b.1 {
            return Ordering::Greater;
        } else if a.1 < b.1 {
            return Ordering::Less;
        } else if a.0 > b.0 {
            return Ordering::Greater;
        } else if a.0 < b.0 {
            return Ordering::Less;
        } else {
            return Ordering::Equal
        }
    });

    let mut int_it = 0;
    while int_it < intersections.len() {
        // fill tiles between pairs of intersections
        let y = intersections[int_it].1;

        let mut x = intersections[int_it].0 + 1;
        while x < intersections[int_it + 1].0 {
            tiles.push((x, y, zoom));

            x = x + 1;
        }

        int_it = int_it + 2;
    }
}

fn poly_cover_single(intersections: &mut Vec<(i32, i32)>, tiles: &mut Vec<(i32, i32, u8)>, linestring: &geo::LineString<f64>, zoom: u8) {
    let mut ring: Vec<(i32, i32)> = Vec::new();

    line_cover(tiles, &linestring, zoom, Some(&mut ring));

    if ring.len() > 0 {
        let mut j = 0;
        let len = ring.len();
        let mut k = len - 1;

        while j < len {
            let m = (j + 1) % len;
            let y = ring[j].1;

            //Add Intersection if it's not local extrenum or Duplicate
            //      Not Local Mim                               Not Local Max
            if (y > ring[k].1 || y > ring[m].1) && (y < ring[k].1 || y < ring[m].1) && y != ring[m].1 {
                intersections.push(ring[j]);
            }

            k = j;
            j = j + 1;
        }
    }
}

pub fn line_cover(tiles: &mut Vec<(i32, i32, u8)>, linestring: &geo::LineString<f64>, zoom: u8, mut ring: Option<&mut Vec<(i32, i32)>>) {
    let mut prev_x: Option<f64> = None;
    let mut prev_y: Option<f64> = None;
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;

    let mut i = 0;
    while i < linestring.0.len() - 1 {
        let start = point_to_tile_fraction(linestring.0[i].x(), linestring.0[i].y(), zoom);
        let stop = point_to_tile_fraction(linestring.0[i + 1].x(), linestring.0[i + 1].y(), zoom);

        let x0 = start.0;
        let y0 = start.1;

        let x1 = stop.0;
        let y1 = stop.1;

        let dx = x1 - x0;
        let dy = y1 - y0;

        if dy == 0.0 && dx == 0.0 {
            i = i + 1;
            continue;
        }

        let sx = if dx > 0.0 { 1.0 } else { -1.0 };
        let sy = if dy > 0.0 { 1.0 } else { -1.0 };

        x = x0.floor();
        y = y0.floor();

        let mut t_max_x = if dx == 0.0 {
            INF
        } else {
            (((if dx > 0.0 { 1.0 } else { 0.0 }) + x - x0) / dx).abs()
        };

        let mut t_max_y = if dy == 0.0 {
            INF
        } else {
            (((if dy > 0.0 { 1.0 } else { 0.0 }) + y - y0) / dy).abs()
        };

        let tdx = (sx / dx).abs();
        let tdy = (sy / dy).abs();

        if Some(x) != prev_x || Some(y) != prev_y {
            tiles.push((x as i32, y as i32, zoom));

            if ring != None && Some(y) != prev_y {
                match ring {
                    Some(ref mut r) => r.push((x as i32, y as i32)),
                    _ => ()
                };
            }

            prev_x = Some(x);
            prev_y = Some(y);
        }

        while t_max_x < 1.0 || t_max_y < 1.0 {
            if t_max_x < t_max_y {
                t_max_x = t_max_x + tdx;
                x = x + sx;
            } else {
                t_max_y = t_max_y + tdy;
                y = y + sy;
            }

            tiles.push((x as i32, y as i32, zoom));

            if ring != None && Some(y) != prev_y {
                match ring {
                    Some(ref mut r) => r.push((x as i32, y as i32)),
                    _ => ()
                };
            }
            prev_x = Some(x);
            prev_y = Some(y);
        }

        i = i + 1;
    }

    if ring != None {
        match ring {
            Some(ref mut r) => {
                if r.len() > 0 && y as i32 == r[0].1 {
                    r.pop();
                }
            },
            _ => ()
        }
    }
}

pub fn get_children(tile: (i32, i32, u8)) -> Vec<(i32, i32, u8)> {
    vec![
        (tile.0 * 2, tile.1 * 2, tile.2 + 1),
        (tile.0 * 2 + 1, tile.1 * 2, tile.2 + 1),
        (tile.0 * 2 + 1, tile.1 * 2 + 1, tile.2 + 1),
        (tile.0 * 2, tile.1 * 2 + 1, tile.2 + 1)
    ]
}

pub fn get_parent(tile: (i32, i32, u8)) -> (i32, i32, u8) {
    (tile.0 >> 1, tile.1 >> 1, tile.2 - 1)
}

pub fn get_siblings(tile: (i32, i32, u8)) -> Vec<(i32, i32, u8)> {
    get_children(get_parent(tile))
}

/**
 * Get the BBOX of a tile
 *
 * Returned in the format [ West, South, East, North ]
 */
pub fn tile_to_bbox(tile: (i32, i32, u8)) -> (f64, f64, f64, f64) {
    (
        tile_to_lon(tile.0, tile.2),
        tile_to_lat(tile.1 + 1, tile.2),
        tile_to_lon(tile.0 + 1, tile.2),
        tile_to_lat(tile.1, tile.2)
    )
}

/**
 * Get the longitudinal value for a given tile corner
 */
pub fn tile_to_lon(x: i32, z: u8) -> f64 {
    x as f64 / (2.0 as f64).powi(z as i32) * 360.0 - 180.0
}


/**
 * Get the latitudinal value for a given tile corner
 */
pub fn tile_to_lat(y: i32, z: u8) -> f64 {
    let n: f64 = PI - 2.0 * PI * y as f64 / (2.0 as f64).powi(z as i32);
    R2D * (0.5 * (n.exp() - (-n).exp())).atan()
}

/**
 *  * Get the tile for a point at a specified zoom level
 */
pub fn point_to_tile(lon: f64, lat: f64, z: u8) -> (i32, i32, u8) {
    let tile_frac = point_to_tile_fraction(lon, lat, z);

    (tile_frac.0.floor() as i32, tile_frac.1.floor() as i32, tile_frac.2)
}

/**
 *  * Get the precise fractional tile location for a point at a zoom level
 */
pub fn point_to_tile_fraction(lon: f64, lat: f64, z: u8) -> (f64, f64, u8) {
    let sin = (lat * D2R).sin();
    let base: f64 = 2.0;

    let z2: f64 = base.powf(z as f64);
    let mut x = z2 * (lon / 360.0 + 0.5);
    let y = z2 * (0.5 - 0.25 * ((1.0 + sin) / (1.0 - sin)).ln() / PI);

    // Wrap Tile X
    x = x % z2;
    if x < 0.0 {
        x = x + z2
    }

    (x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let point = Point::new(-77.15664982795715, 38.87419791355846);
        let geom = point.into();
        assert_eq!(tiles(&geom, 1).unwrap(), vec![ (1, 1, 1) ]);
        assert_eq!(tiles(&geom, 2).unwrap(), vec![ (2, 3, 2) ]);
        assert_eq!(tiles(&geom, 3).unwrap(), vec![ (4, 6, 3) ]);
        assert_eq!(tiles(&geom, 4).unwrap(), vec![ (9, 13, 4) ]);
    }

    #[test]
    fn test_points() {
        let points: MultiPoint<f64> = vec![
            ( -84.48486328124999, 43.40504748787035 ),
            ( -90.87890625, 39.90973623453719 ),
            ( -84.55078125, 43.45291889355468 ),
            ( -90.8349609375, 39.93711893299021 )
        ].into();
        let geom = points.into();
        assert_eq!(tiles(&geom, 1).unwrap(), vec![ (1, 1, 1), (1, 2, 1) ]);
        assert_eq!(tiles(&geom, 2).unwrap(), vec![ (2, 3, 2), (2, 5, 2) ]);
        assert_eq!(tiles(&geom, 3).unwrap(), vec![ (4, 7, 3), (4, 10, 3) ]);
        assert_eq!(tiles(&geom, 4).unwrap(), vec![ (9, 15, 4), (9, 20, 4) ]);
    }

    #[test]
    fn test_line() {
        let line = LineString(vec![
            Point::new(-106.21719360351562, 28.592359801121567),
            Point::new(-106.1004638671875, 28.791130513231813),
            Point::new(-105.87661743164062, 28.864519767126602),
            Point::new(-105.82374572753905, 28.60743139267596)
        ]);

        let geom = line.into();
        assert_eq!(tiles(&geom, 12).unwrap(), vec![
            ( 839, 1707, 12 ),
            ( 839, 1708, 12 ),
            ( 840, 1705, 12 ),
            ( 840, 1706, 12 ),
            ( 840, 1707, 12 ),
            ( 841, 1705, 12 ),
            ( 842, 1704, 12 ),
            ( 842, 1705, 12 ),
            ( 843, 1704, 12 ),
            ( 843, 1705, 12 ),
            ( 843, 1706, 12 ),
            ( 843, 1707, 12 ),
            ( 843, 1708, 12 )
        ])
    }

    #[test]
    fn test_edge_line() {
        let line = LineString(vec![
            Point::new(-80.160384, 32.766901),
            Point::new(-80.160216, 32.766845),
            Point::new(-80.159659, 32.766722),
            Point::new(-80.159356, 32.766633),
            Point::new(-80.159196, 32.766586),
            Point::new(-80.159096, 32.766571),
            Point::new(-80.159016, 32.766569),
            Point::new(-80.158947, 32.766581),
            Point::new(-80.158637, 32.766668),
            Point::new(-80.158527, 32.766691),
            Point::new(-80.158433, 32.766697),
            Point::new(-80.158367, 32.76669),
            Point::new(-80.158116, 32.766641),
            Point::new(-80.157565, 32.766507),
            Point::new(-80.157183, 32.766389),
            Point::new(-80.156946, 32.76633),
            Point::new(-80.156748, 32.766298),
            Point::new(-80.156657, 32.766279),
            Point::new(-80.156492, 32.766253),
            Point::new(-80.15626, 32.766181),
            Point::new(-80.156216, 32.766155),
            Point::new(-80.156166, 32.766118),
            Point::new(-80.156148, 32.7661),
            Point::new(-80.156125, 32.766052),
            Point::new(-80.156122, 32.766012),
            Point::new(-80.156131, 32.765974),
            Point::new(-80.156179, 32.765905),
            Point::new(-80.156198, 32.765856),
            Point::new(-80.15621, 32.765807),
            Point::new(-80.15625, 32.76548),
            Point::new(-80.156249, 32.765323),
            Point::new(-80.156235, 32.765284),
            Point::new(-80.156215, 32.765256),
            Point::new(-80.156181, 32.765226)
        ]);

        let geom = line.into();
        assert_eq!(tiles(&geom, 14).unwrap(), vec![
            (4543, 6612, 14),
            (4544, 6612, 14)
        ])
    }

    #[test]
    fn test_multiline() {
        let line = MultiLineString(vec![
            LineString(vec![
                Point::new(11.3818359375, 51.15178610143037),
                Point::new(7.998046875, 50.0077390146369),
                Point::new(10.458984375, 49.18170338770663),
                Point::new(5.2734375, 46.6795944656402),
            ]),
            LineString(vec![
                Point::new(0.263671875, 49.15296965617042),
                Point::new(3.076171875, 50.0077390146369),
                Point::new(3.6474609374999996, 48.60385760823255),
                Point::new(4.7900390625, 49.095452162534826),
                Point::new(6.328125, 48.48748647988415),
                Point::new(10.1513671875, 48.07807894349862),
                Point::new(12.392578125, 46.46813299215554),
            ])
        ]);

        let geom = line.into();
        assert_eq!(tiles(&geom, 8).unwrap(), vec![
            ( 128, 87, 8 ),
            ( 129, 86, 8 ),
            ( 129, 87, 8 ),
            ( 130, 86, 8 ),
            ( 130, 87, 8 ),
            ( 130, 88, 8 ),
            ( 131, 87, 8 ),
            ( 131, 88, 8 ),
            ( 131, 90, 8 ),
            ( 132, 88, 8 ),
            ( 132, 89, 8 ),
            ( 132, 90, 8 ),
            ( 133, 86, 8 ),
            ( 133, 88, 8 ),
            ( 133, 89, 8 ),
            ( 134, 86, 8 ),
            ( 134, 87, 8 ),
            ( 134, 88, 8 ),
            ( 135, 85, 8 ),
            ( 135, 86, 8 ),
            ( 135, 87, 8 ),
            ( 135, 88, 8 ),
            ( 135, 89, 8 ),
            ( 136, 85, 8 ),
            ( 136, 89, 8 ),
            ( 136, 90, 8 )
        ]);
    }

    #[test]
    fn test_polygon() {
        let poly = Polygon::new(
            LineString(vec![
                Point::new(5.11962890625, 20.46818922264095),
                Point::new(5.11962890625, 20.7663868125152),
                Point::new(5.504150390625, 20.7663868125152),
                Point::new(5.504150390625, 20.46818922264095),
                Point::new(5.11962890625, 20.46818922264095),
            ]),
            Vec::<LineString<f64>>::new()
        );

        let geom = poly.into();
        assert_eq!(tiles(&geom, 8).unwrap(), vec![
             ( 131, 112, 8 ),
             ( 131, 113, 8 )
        ]);
    }

    #[test]
    fn test_polygon_building() {
        let poly = Polygon::new(
            LineString(vec![
                Point::new(-77.15269088745116,38.87153962460514),
                Point::new(-77.1521383523941,38.871322446566325),
                Point::new(-77.15196132659912,38.87159391901113),
                Point::new(-77.15202569961546,38.87162315444336),
                Point::new(-77.1519023180008,38.87179021382536),
                Point::new(-77.15266406536102,38.8727758561868),
                Point::new(-77.1527713537216,38.87274662122871),
                Point::new(-77.15282499790192,38.87282179681094),
                Point::new(-77.15323269367218,38.87267562199469),
                Point::new(-77.15313613414764,38.87254197618533),
                Point::new(-77.15270698070526,38.87236656567917),
                Point::new(-77.1523904800415,38.87198233162923),
                Point::new(-77.15269088745116,38.87153962460514),
            ]),
            Vec::<LineString<f64>>::new()
        );

        let geom = poly.into();
        assert_eq!(tiles(&geom, 18).unwrap(), vec![
            ( 74890, 100305, 18 ),
            ( 74891, 100305, 18 ),
            ( 74891, 100306, 18 )
        ]);
    }

    #[test]
    fn test_polygon_donut() {
        let poly = Polygon::new(
            LineString(vec![
                Point::new(-76.165286,45.479514),
                Point::new(-76.140095,45.457437),
                Point::new(-76.162348,45.444872),
                Point::new(-76.168656,45.441087),
                Point::new(-76.201963,45.420225),
                Point::new(-76.213668,45.429276),
                Point::new(-76.214261,45.429917),
                Point::new(-76.227477,45.440383),
                Point::new(-76.263056,45.467983),
                Point::new(-76.245084,45.468609),
                Point::new(-76.240206,45.471202),
                Point::new(-76.238518,45.475254),
                Point::new(-76.233483,45.507829),
                Point::new(-76.227816,45.511836),
                Point::new(-76.212117,45.51623),
                Point::new(-76.191776,45.50154),
                Point::new(-76.174016,45.486911),
                Point::new(-76.165286,45.479514)
            ]),
            vec![LineString(vec![
                Point::new(-76.227618, 45.489247),
                Point::new(-76.232113, 45.486983),
                Point::new(-76.232151, 45.486379),
                Point::new(-76.231812, 45.485106),
                Point::new(-76.230698, 45.483236),
                Point::new(-76.225664, 45.477365),
                Point::new(-76.223568, 45.475174),
                Point::new(-76.202829, 45.458815),
                Point::new(-76.200229, 45.458822),
                Point::new(-76.199069, 45.459164),
                Point::new(-76.188361, 45.465784),
                Point::new(-76.204505, 45.479018),
                Point::new(-76.215555, 45.488534),
                Point::new(-76.220249, 45.492175),
                Point::new(-76.221154, 45.493315),
                Point::new(-76.22631, 45.490189),
                Point::new(-76.226543, 45.489754),
                Point::new(-76.227618, 45.489247)
            ])]
        );

        let geom = poly.into();
        assert_eq!(tiles(&geom, 16).unwrap(), vec![
            (18884, 23453, 16),
            (18884, 23454, 16),
            (18885, 23453, 16),
            (18885, 23454, 16),
            (18885, 23455, 16),
            (18886, 23453, 16),
            (18886, 23454, 16),
            (18886, 23455, 16),
            (18886, 23456, 16),
            (18887, 23453, 16),
            (18887, 23454, 16),
            (18887, 23455, 16),
            (18887, 23456, 16),
            (18887, 23457, 16),
            (18888, 23452, 16),
            (18888, 23453, 16),
            (18888, 23454, 16),
            (18888, 23455, 16),
            (18888, 23456, 16),
            (18888, 23457, 16),
            (18888, 23458, 16),
            (18889, 23444, 16),
            (18889, 23445, 16),
            (18889, 23446, 16),
            (18889, 23447, 16),
            (18889, 23448, 16),
            (18889, 23449, 16),
            (18889, 23450, 16),
            (18889, 23451, 16),
            (18889, 23452, 16),
            (18889, 23453, 16),
            (18889, 23454, 16),
            (18889, 23455, 16),
            (18889, 23456, 16),
            (18889, 23457, 16),
            (18889, 23458, 16),
            (18889, 23459, 16),
            (18890, 23442, 16),
            (18890, 23443, 16),
            (18890, 23444, 16),
            (18890, 23445, 16),
            (18890, 23446, 16),
            (18890, 23447, 16),
            (18890, 23448, 16),
            (18890, 23449, 16),
            (18890, 23450, 16),
            (18890, 23451, 16),
            (18890, 23452, 16),
            (18890, 23453, 16),
            (18890, 23454, 16),
            (18890, 23455, 16),
            (18890, 23456, 16),
            (18890, 23457, 16),
            (18890, 23458, 16),
            (18890, 23459, 16),
            (18890, 23460, 16),
            (18891, 23442, 16),
            (18891, 23443, 16),
            (18891, 23444, 16),
            (18891, 23445, 16),
            (18891, 23446, 16),
            (18891, 23447, 16),
            (18891, 23448, 16),
            (18891, 23450, 16),
            (18891, 23451, 16),
            (18891, 23452, 16),
            (18891, 23453, 16),
            (18891, 23454, 16),
            (18891, 23455, 16),
            (18891, 23456, 16),
            (18891, 23457, 16),
            (18891, 23458, 16),
            (18891, 23459, 16),
            (18891, 23460, 16),
            (18891, 23461, 16),
            (18891, 23462, 16),
            (18892, 23441, 16),
            (18892, 23442, 16),
            (18892, 23443, 16),
            (18892, 23444, 16),
            (18892, 23445, 16),
            (18892, 23446, 16),
            (18892, 23447, 16),
            (18892, 23448, 16),
            (18892, 23452, 16),
            (18892, 23453, 16),
            (18892, 23454, 16),
            (18892, 23455, 16),
            (18892, 23456, 16),
            (18892, 23457, 16),
            (18892, 23458, 16),
            (18892, 23459, 16),
            (18892, 23460, 16),
            (18892, 23461, 16),
            (18892, 23462, 16),
            (18892, 23463, 16),
            (18893, 23441, 16),
            (18893, 23442, 16),
            (18893, 23443, 16),
            (18893, 23444, 16),
            (18893, 23445, 16),
            (18893, 23446, 16),
            (18893, 23447, 16),
            (18893, 23448, 16),
            (18893, 23449, 16),
            (18893, 23453, 16),
            (18893, 23454, 16),
            (18893, 23455, 16),
            (18893, 23456, 16),
            (18893, 23457, 16),
            (18893, 23458, 16),
            (18893, 23459, 16),
            (18893, 23460, 16),
            (18893, 23461, 16),
            (18893, 23462, 16),
            (18893, 23463, 16),
            (18893, 23464, 16),
            (18894, 23441, 16),
            (18894, 23442, 16),
            (18894, 23443, 16),
            (18894, 23444, 16),
            (18894, 23445, 16),
            (18894, 23446, 16),
            (18894, 23447, 16),
            (18894, 23448, 16),
            (18894, 23449, 16),
            (18894, 23450, 16),
            (18894, 23454, 16),
            (18894, 23455, 16),
            (18894, 23456, 16),
            (18894, 23457, 16),
            (18894, 23458, 16),
            (18894, 23459, 16),
            (18894, 23460, 16),
            (18894, 23461, 16),
            (18894, 23462, 16),
            (18894, 23463, 16),
            (18894, 23464, 16),
            (18894, 23465, 16),
            (18895, 23442, 16),
            (18895, 23443, 16),
            (18895, 23444, 16),
            (18895, 23445, 16),
            (18895, 23446, 16),
            (18895, 23447, 16),
            (18895, 23448, 16),
            (18895, 23449, 16),
            (18895, 23450, 16),
            (18895, 23451, 16),
            (18895, 23455, 16),
            (18895, 23456, 16),
            (18895, 23457, 16),
            (18895, 23458, 16),
            (18895, 23459, 16),
            (18895, 23460, 16),
            (18895, 23461, 16),
            (18895, 23462, 16),
            (18895, 23463, 16),
            (18895, 23464, 16),
            (18895, 23465, 16),
            (18895, 23466, 16),
            (18896, 23443, 16),
            (18896, 23444, 16),
            (18896, 23445, 16),
            (18896, 23446, 16),
            (18896, 23447, 16),
            (18896, 23448, 16),
            (18896, 23449, 16),
            (18896, 23450, 16),
            (18896, 23451, 16),
            (18896, 23452, 16),
            (18896, 23455, 16),
            (18896, 23456, 16),
            (18896, 23457, 16),
            (18896, 23458, 16),
            (18896, 23459, 16),
            (18896, 23460, 16),
            (18896, 23461, 16),
            (18896, 23462, 16),
            (18896, 23463, 16),
            (18896, 23464, 16),
            (18896, 23465, 16),
            (18896, 23466, 16),
            (18897, 23444, 16),
            (18897, 23445, 16),
            (18897, 23446, 16),
            (18897, 23447, 16),
            (18897, 23448, 16),
            (18897, 23449, 16),
            (18897, 23450, 16),
            (18897, 23451, 16),
            (18897, 23452, 16),
            (18897, 23453, 16),
            (18897, 23454, 16),
            (18897, 23455, 16),
            (18897, 23456, 16),
            (18897, 23457, 16),
            (18897, 23458, 16),
            (18897, 23459, 16),
            (18897, 23460, 16),
            (18897, 23461, 16),
            (18897, 23462, 16),
            (18897, 23463, 16),
            (18897, 23464, 16),
            (18897, 23465, 16),
            (18898, 23445, 16),
            (18898, 23446, 16),
            (18898, 23447, 16),
            (18898, 23448, 16),
            (18898, 23449, 16),
            (18898, 23450, 16),
            (18898, 23451, 16),
            (18898, 23452, 16),
            (18898, 23453, 16),
            (18898, 23454, 16),
            (18898, 23455, 16),
            (18898, 23456, 16),
            (18898, 23457, 16),
            (18898, 23458, 16),
            (18898, 23459, 16),
            (18898, 23460, 16),
            (18898, 23461, 16),
            (18898, 23462, 16),
            (18898, 23463, 16),
            (18898, 23464, 16),
            (18899, 23446, 16),
            (18899, 23447, 16),
            (18899, 23448, 16),
            (18899, 23449, 16),
            (18899, 23450, 16),
            (18899, 23451, 16),
            (18899, 23452, 16),
            (18899, 23453, 16),
            (18899, 23454, 16),
            (18899, 23455, 16),
            (18899, 23456, 16),
            (18899, 23457, 16),
            (18899, 23458, 16),
            (18899, 23459, 16),
            (18899, 23460, 16),
            (18899, 23461, 16),
            (18899, 23462, 16),
            (18899, 23463, 16),
            (18900, 23447, 16),
            (18900, 23448, 16),
            (18900, 23449, 16),
            (18900, 23450, 16),
            (18900, 23451, 16),
            (18900, 23452, 16),
            (18900, 23453, 16),
            (18900, 23454, 16),
            (18900, 23455, 16),
            (18900, 23456, 16),
            (18900, 23457, 16),
            (18900, 23458, 16),
            (18900, 23459, 16),
            (18900, 23460, 16),
            (18900, 23461, 16),
            (18900, 23462, 16),
            (18901, 23449, 16),
            (18901, 23450, 16),
            (18901, 23451, 16),
            (18901, 23452, 16),
            (18901, 23453, 16),
            (18901, 23454, 16),
            (18901, 23455, 16),
            (18901, 23456, 16),
            (18901, 23457, 16),
            (18901, 23458, 16),
            (18901, 23459, 16),
            (18901, 23460, 16),
            (18901, 23461, 16),
            (18902, 23450, 16),
            (18902, 23451, 16),
            (18902, 23452, 16),
            (18902, 23453, 16),
            (18902, 23454, 16),
            (18902, 23455, 16),
            (18902, 23456, 16),
            (18902, 23457, 16),
            (18902, 23458, 16),
            (18902, 23459, 16),
            (18902, 23460, 16),
            (18903, 23451, 16),
            (18903, 23452, 16),
            (18903, 23453, 16),
            (18903, 23454, 16),
            (18903, 23455, 16),
            (18903, 23456, 16),
            (18903, 23457, 16),
            (18903, 23458, 16),
            (18903, 23459, 16),
            (18903, 23460, 16),
            (18904, 23452, 16),
            (18904, 23453, 16),
            (18904, 23454, 16),
            (18904, 23455, 16),
            (18904, 23456, 16),
            (18904, 23457, 16),
            (18904, 23458, 16),
            (18904, 23459, 16),
            (18905, 23454, 16),
            (18905, 23455, 16),
            (18905, 23456, 16),
            (18905, 23457, 16),
            (18905, 23458, 16),
            (18906, 23455, 16),
            (18906, 23456, 16),
            (18906, 23457, 16),
            (18907, 23456, 16)
        ]);
    }

    #[test]
    fn test_get_parent() {
        assert_eq!(get_parent((5, 10, 10)), (2, 5, 9))
    }

    #[test]
    fn test_get_siblings() {
        assert_eq!(get_siblings((5, 10, 10)), vec![(4, 10, 10), (5, 10, 10), (5, 11, 10), (4, 11, 10)])
    }

    #[test]
    fn test_tile_to_bbox() {
        assert_eq!(tile_to_bbox((5, 10, 10)), (-178.2421875, 84.7060489350415, -177.890625, 84.73838712095339));
    }

    #[test]
    fn test_point_to_tile_fraction() {
         assert_eq!(point_to_tile_fraction(-95.93965530395508, 41.26000108568697, 9), (119.552490234375, 191.47119140625, 9));
    }

    #[test]
    fn test_point_to_tile() {
        assert_eq!(point_to_tile(0.0, 0.0, 10), (512, 512, 10));
        assert_eq!(point_to_tile(-77.03239381313323, 38.91326516559442, 10), (292, 391, 10));
    }

    #[test]
    fn test_point_to_tile_cross_meridian_x() {
        assert_eq!(point_to_tile(-180.0, 0.0, 0), (0, 0, 0));
        assert_eq!(point_to_tile(-180.0, 85.0, 2), (0, 0, 2));
        assert_eq!(point_to_tile(180.0, 85.0, 2), (0, 0, 2));
        assert_eq!(point_to_tile(-185.0, 85.0, 2), (3, 0, 2));
        assert_eq!(point_to_tile(185.0, 85.0, 2), (0, 0, 2));
    }

    #[test]
    fn test_point_to_tile_cross_meridian_y() {
        assert_eq!(point_to_tile(-175.0, -95.0, 2), (0, 3, 2));
        assert_eq!(point_to_tile(-175.0, 95.0, 2), (0, 0, 2));
    }
}
