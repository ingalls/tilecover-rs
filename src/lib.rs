extern crate postgis;

use std::f64::consts::PI;
use postgis::ewkb::Geometry;

const D2R: f64 = PI / 180.0;
const _R2D: f64 = 180.0 / PI;

pub fn tiles(geom: Geometry, _limits: Option<(u8, u8)>) -> Vec<(i32, i32, u8)> {

    vec!((10, 10, 1))
}

/**
 *  * Get the tile for a point at a specified zoom level
 */
pub fn point_to_tile(lon: f64, lat: f64, z: u8) -> (i32, i32, u8) {
    let tile_frac = point_to_tile_fraction(lon, lat, z);

    let tile: (i32, i32, u8) = (tile_frac.0.floor() as i32, tile_frac.1.floor() as i32, tile_frac.2);

    tile
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
