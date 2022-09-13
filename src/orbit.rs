// special thanks to nchashch who provided https://github.com/nchashch/orbital-mechanics-rust
// on which this was originally based

// TODO: try https://space.stackexchange.com/questions/15366/how-do-you-model-hyperbolic-orbits

use glam::{dvec3, DMat3, DVec3};
use std::{f64::consts::PI, ops::Div};

/// # Keplerian Orbital Elements
///
/// This structure represents an orbit using
/// six keplerian element. It also holds mean motion
/// to avoid recomputing it for every tick() call, and
/// rot matrix to avoid recomputing it for every CSV::from_koe() call.
/// Like CSV it holds a reference to the central body.
#[derive(Clone, Debug)]
pub struct Koe {
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    /// Longitude of Ascending Node.
    pub lan: f64,
    /// Argument of Periapsis.
    pub ap: f64,
    pub mean_anomaly: f64,
}

impl Koe {
    /// mu is a standard gravitational parameter Mass * Universal gravitational constant
    pub fn period(&self, mu: f64) -> f64 {
        self.semi_major_axis.powf(3.0).div(mu).sqrt() * PI * 2.0
    }

    /// mu is a standard gravitational parameter Mass * Universal gravitational constant
    pub fn tick(&self, dt: f64, mu: f64) -> Self {
        let mean_motion = (mu / self.semi_major_axis.powf(3.0)).sqrt();
        Koe {
            mean_anomaly: self.mean_anomaly + mean_motion * dt,
            ..*self
        }
    }

    fn rot(&self) -> DMat3 {
        DMat3::from_axis_angle(dvec3(0.0, 0.0, 1.0), self.ap)
    }

    /// Construct KOE from CSV.
    ///
    /// mu is a standard gravitational parameter: Mass of parent body * Universal gravitational constant
    pub fn from_csv(csv: Csv, mu: f64) -> Koe {
        let position = csv.pos;
        let velocity = csv.vel;

        let specific_angular_momentum = position.cross(velocity);

        // Eccentricity vector
        // (It is pointed towards periapsis and it's length
        // is equal to eccentricity of the orbit)
        let ev = velocity.cross(specific_angular_momentum) / mu - position.normalize();
        let eccentricity = ev.length();
        dbg!(eccentricity);

        // Node vector
        // (vector pointing towards the ascending node)
        // Ascending node is a point where satelite
        // is above equator and goes north
        let acending_node = DVec3::Z.cross(specific_angular_momentum);

        let cos_inc = specific_angular_momentum.dot(DVec3::Z) / specific_angular_momentum.length();
        // cos_inc is sometimes greater than 1.0
        // and without this fix cos_inc.acos() is NaN
        // for cos_inc > 1.0 cases
        let inclination = if cos_inc > 1.0 { 0.0 } else { cos_inc.acos() };

        // Longitude of Ascending Node
        // (angle between vector csv.cb.i and ascending node)
        let mut lan = if acending_node.dot(DVec3::NEG_Y) >= 0.0 {
            (acending_node.dot(DVec3::X) / acending_node.length()).acos()
        } else {
            2.0 * PI - (acending_node.dot(DVec3::X) / acending_node.length()).acos()
        };

        let right = specific_angular_momentum.cross(acending_node);
        // Argument of periapsis
        // (angle between ascending node and periapsis)
        let mut ap = if ev.dot(right) >= 0.0 {
            (acending_node.dot(ev) / (acending_node.length() * ev.length())).acos()
        } else {
            2.0 * PI - (acending_node.dot(ev) / (acending_node.length() * ev.length())).acos()
        };

        // If the orbit is circular ap is 0.0
        // (ap doesn't make sense for circular orbits)
        if approx_eq(eccentricity, 0.0) {
            ap = 0.0;
        }
        // If the orbit is equatorial lan is 0.0
        // (lan doesn't make sense for equatorial orbits)
        if approx_eq(inclination, 0.0) {
            lan = 0.0;
            // If it is equatorial, non circular orbit ap is Longitude of Periapsis
            // (angle between vector csv.cb.i and periapsis)
            if !approx_eq(eccentricity, 0.0) {
                ap = if ev.dot(DVec3::NEG_Y) >= 0.0 {
                    (DVec3::X.dot(ev) / ev.length()).acos()
                } else {
                    2.0 * PI - (DVec3::X.dot(ev) / ev.length()).acos()
                };
            }
        }

        // True anomaly
        // (angle between periapsis and radius vector)
        let mut ta = if position.dot(velocity) >= 0.0 {
            (ev.dot(position) / (ev.length() * position.length())).acos()
        } else {
            2.0 * PI - (ev.dot(position) / (ev.length() * position.length())).acos()
        };

        if approx_eq(eccentricity, 0.0) {
            // For circular equatorial orbit use longitude
            // (angle between vector csv.cb.i and radius vector)
            if approx_eq(inclination, 0.0) {
                ta = if DVec3::X.dot(velocity) <= 0.0 {
                    (DVec3::X.dot(position) / (DVec3::X.length() * position.length())).acos()
                } else {
                    2.0 * PI
                        - (DVec3::X.dot(position) / (DVec3::X.length() * position.length())).acos()
                }
            // For circular non equatorial orbit use argument of latitude
            // (angle between ascending node and radius vector)
            } else {
                ta = if acending_node.dot(velocity) <= 0.0 {
                    (acending_node.dot(position) / (acending_node.length() * position.length()))
                        .acos()
                } else {
                    2.0 * PI
                        - (acending_node.dot(position)
                            / (acending_node.length() * position.length()))
                        .acos()
                }
            }
        }

        let aa = ((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt();
        debug_assert!(!aa.is_nan());
        // Eccentric anomaly (intermidiate step to compute mean anomaly)
        let ea = 2.0 * ((ta / 2.0).tan() / aa).atan();
        debug_assert!(!ea.is_nan());
        // Mean anomaly (it is used because it changes linearly with time,
        // and for that reason is cheap to update)
        let mean_anomaly = ea - eccentricity * ea.sin();
        debug_assert!(!mean_anomaly.is_nan());

        let semi_major_axis = 1.0 / (2.0 / position.length() - velocity.length_squared() / mu);
        debug_assert!(!semi_major_axis.is_nan());

        Koe {
            semi_major_axis,
            eccentricity,
            inclination,
            lan,
            ap,
            mean_anomaly,
        }
    }
}

/// # Cartesian State Vectors
/// This structure represents an orbit using a
/// radius vector and a velocity vector.
/// It holds a reference to the central body.
#[derive(Clone, Copy, Debug)]
pub struct Csv {
    /// aka radius vector.
    pub pos: DVec3,
    /// Velocity.
    pub vel: DVec3,
}

impl Csv {
    /// Construct CSV from position and velocity.
    pub fn new(r: DVec3, v: DVec3) -> Csv {
        Csv { pos: r, vel: v }
    }

    /// Construct CSV from KOE.
    /// mu is a standard gravitational parameter Mass * Universal gravitational constant
    pub fn from_koe(koe: Koe, mu: f64) -> Csv {
        // Mean anomaly
        let m0 = koe.mean_anomaly;
        // Number of iterations for newton_raphson
        let iterations = 10;
        // Eccentric anomaly
        let ea = Csv::newton_raphson(&m0, &koe.eccentricity, &iterations);
        // True anomaly
        let ta = 2.0
            * ((1.0 + koe.eccentricity).sqrt() * (ea / 2.0).sin())
                .atan2((1.0 - koe.eccentricity).sqrt() * (ea / 2.0).cos());
        // Distance to the center of the central body
        let dist = koe.semi_major_axis * (1.0 - koe.eccentricity * ea.cos());
        // Radius vector in i, j plane
        let mut r = (DVec3::X * ta.cos() + DVec3::NEG_Y * ta.sin()) * dist;
        // Velocity in i, j plane
        let mut v = (DVec3::X * (-ea.sin())
            + DVec3::NEG_Y * ((1.0 - koe.eccentricity.powf(2.0)).sqrt() * ea.cos()))
            * ((mu * koe.semi_major_axis).sqrt() / dist);
        // Radius vector in orbital plane
        r = koe.rot() * r;
        // Velocity in orbital plane
        v = koe.rot() * v;
        Csv::new(r, v)
    }

    // Function that numerically solves Kepler's equation
    fn newton_raphson(m0: &f64, e: &f64, iterations: &i32) -> f64 {
        let mut ea = m0.clone();
        for _ in 0..*iterations {
            ea = ea - (ea - e * ea.sin() - m0) / (1.0 - e * ea.cos());
        }
        ea
    }
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.0000001
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    use super::*;

    #[test]
    #[ignore]
    fn period() {
        const PARENT_MASS: f64 = 5.97219_e15;
        const GRAVITATIONAL_CONSTANT: f64 = 6.67_e-11;
        const MU: f64 = PARENT_MASS * GRAVITATIONAL_CONSTANT;
        const SCALE: f64 = 1000.0;

        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let mut randf = move || rng.gen::<f64>() * SCALE;

        for _ in 0..100 {
            let csv = Csv {
                pos: dvec3(randf(), randf(), randf()),
                vel: dvec3(randf(), randf(), randf()),
            };
            let koe = Koe::from_csv(csv, MU);
            let new_csv = Csv::from_koe(koe.tick(koe.period(MU), MU), MU);
            dbg!(csv, new_csv, koe.period(MU));
            assert!(approx_eq((csv.pos - new_csv.pos).length(), 0.0));
            assert!(approx_eq((csv.vel - new_csv.vel).length(), 0.0));
        }
    }
}
