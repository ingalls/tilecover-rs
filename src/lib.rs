extern crate geo;

use std::f64::consts::PI;
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
            Ok(vec!((10, 10, 1)))
        },
        &geo::Geometry::MultiLineString(ref linestrings) => {
            Ok(vec!((10, 10, 1)))
        },
        &geo::Geometry::Polygon(ref polygon) => {
            Ok(vec!((10, 10, 1)))
        },
        &geo::Geometry::MultiPolygon(ref polygons) => {
            Ok(vec!((10, 10, 1)))
        },
        _ => Err(Error::GeomTypeNotSupported)
    }
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
        let points: MultiPoint<_> = vec![
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
