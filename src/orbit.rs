// special thanks to nchashch who provided https://github.com/nchashch/orbital-mechanics-rust
// on which this was originally based

use macroquad::prelude::glam;
use nalgebra::*;
use std::f64::consts::*;
use std::rc::*;

/// # Keplerian Orbital Elements
/// This structure represents an orbit using
/// six keplerian element. It also holds mean motion
/// to avoid recomputing it for every tick() call, and
/// rot matrix to avoid recomputing it for every CSV::from_koe() call.
/// Like CSV it holds a reference to the central body.
#[derive(Clone)]
pub struct Koe {
    /// Semi Major Axis.
    pub a: f64,
    /// Eccentricity.
    pub e: f64,
    /// Inclination.
    pub inc: f64,
    /// Longitude of Ascending Node.
    pub lan: f64,
    /// Argument of Periapsis.
    pub ap: f64,
    /// Mean anomaly.
    pub m0: f64,
    /// Mean motion.
    pub n: f64,
    /// A matrix that transforms a vector lying in i, j plane into a corresponding vector lying in the orbital plane
    /// defined by inc and lan, it is stored in KOE to avoid recomputing it for every CSV::from_koe() call.
    pub rot: Rot3<f64>,
    /// Reference to the central body.
    pub cb: Rc<CB>,
}

impl Koe {
    pub fn tick(&self, dt: f64) -> Self {
        Koe {
            m0: self.m0 + self.n * dt,
            cb: self.cb.clone(),
            ..*self
        }
    }

    /// Construct KOE from orbital elements and CB reference.
    pub fn new(a: f64, e: f64, inc: f64, lan: f64, ap: f64, m0: f64, cb: Rc<CB>) -> Koe {
        // Vector cb.i points towards intersection of 0th meridian and equator
        // Vector cb.k points towards north pole
        // cb.j == cross(&cb.k, &cb.i)

        // rot transformation matrix is not expected to change very often
        // and it is stored in KOE to avoid recomputing it
        // for every CSV::from_koe() call
        let mut rot = Rot3::new_identity(3);
        // Do lan and inc rotations only if
        // the orbit is not equatorial
        if !approx_eq(&inc, &0.0) {
            let lan_axisangle = cb.k * lan;
            let inc_axisangle = cb.i * inc;
            rot = rot.prepend_rotation(&lan_axisangle);
            rot = rot.prepend_rotation(&inc_axisangle);
        }
        // Do ap rotation only if
        // the orbit is not circular
        if !approx_eq(&e, &0.0) {
            let ap_axisangle = cb.k * ap;
            rot = rot.prepend_rotation(&ap_axisangle);
        }
        // Mean motion
        let n = (cb.mu / a.powf(3.0)).sqrt();
        Koe {
            a,
            e,
            inc,
            lan,
            ap,
            m0,
            n,
            rot,
            cb,
        }
    }

    /// Construct KOE from CSV.
    pub fn from_csv(csv: Csv) -> Koe {
        // Vector csv.cb.i points towards intersection of 0th meridian and equator
        // Vector csv.cb.k points towards north pole
        // csv.cb.j == cross(&csv.cb.k, &csv.cb.i)

        // Radius vector
        let r = csv.r;
        // Velocity
        let v = csv.v;

        // Standard gravitational parameter
        let mu = &csv.cb.mu;

        // Specific angular momentum
        let h = cross(&r, &v);

        // Eccentricity vector
        // (It is pointed towards periapsis and it's length
        // is equal to eccentricity of the orbit)
        let e = cross(&v, &h) / *mu - normalize(&r);

        // Node vector
        // (vector pointing towards the ascending node)
        // Ascending node is a point where satelite
        // is above equator and goes north
        let n = cross(&csv.cb.k, &h);

        let cos_inc = dot(&h, &csv.cb.k) / (norm(&h));
        // cos_inc is sometimes greater than 1.0
        // and without this fix cos_inc.acos() is NaN
        // for cos_inc > 1.0 cases
        let inc = if cos_inc > 1.0 { 0.0 } else { cos_inc.acos() };

        // Eccentricity
        let es = norm(&e);

        // Longitude of Ascending Node
        // (angle between vector csv.cb.i and ascending node)
        let mut lan = if dot(&n, &csv.cb.j) >= 0.0 {
            (dot(&n, &csv.cb.i) / norm(&n)).acos()
        } else {
            2.0 * PI - (dot(&n, &csv.cb.i) / norm(&n)).acos()
        };

        let right = cross(&h, &n);
        // Argument of periapsis
        // (angle between ascending node and periapsis)
        let mut ap = if dot(&e, &right) >= 0.0 {
            (dot(&n, &e) / (norm(&n) * norm(&e))).acos()
        } else {
            2.0 * PI - (dot(&n, &e) / (norm(&n) * norm(&e))).acos()
        };

        // If the orbit is circular ap is 0.0
        // (ap doesn't make sense for circular orbits)
        if approx_eq(&es, &0.0) {
            ap = 0.0;
        }
        // If the orbit is equatorial lan is 0.0
        // (lan doesn't make sense for equatorial orbits)
        if approx_eq(&inc, &0.0) {
            lan = 0.0;
            // If it is equatorial, non circular orbit ap is Longitude of Periapsis
            // (angle between vector csv.cb.i and periapsis)
            if !approx_eq(&es, &0.0) {
                ap = if dot(&e, &csv.cb.j) >= 0.0 {
                    (dot(&csv.cb.i, &e) / norm(&e)).acos()
                } else {
                    2.0 * PI - (dot(&csv.cb.i, &e) / norm(&e)).acos()
                };
            }
        }

        // True anomaly
        // (angle between periapsis and radius vector)
        let mut ta = if dot(&r, &v) >= 0.0 {
            (dot(&e, &r) / (norm(&e) * norm(&r))).acos()
        } else {
            2.0 * PI - (dot(&e, &r) / (norm(&e) * norm(&r))).acos()
        };

        if approx_eq(&es, &0.0) {
            // For circular equatorial orbit use longitude
            // (angle between vector csv.cb.i and radius vector)
            if approx_eq(&inc, &0.0) {
                ta = if dot(&csv.cb.i, &v) <= 0.0 {
                    (dot(&csv.cb.i, &r) / (norm(&csv.cb.i) * norm(&r))).acos()
                } else {
                    2.0 * PI - (dot(&csv.cb.i, &r) / (norm(&csv.cb.i) * norm(&r))).acos()
                }
            // For circular non equatorial orbit use argument of latitude
            // (angle between ascending node and radius vector)
            } else {
                ta = if dot(&n, &v) <= 0.0 {
                    (dot(&n, &r) / (norm(&n) * norm(&r))).acos()
                } else {
                    2.0 * PI - (dot(&n, &r) / (norm(&n) * norm(&r))).acos()
                }
            }
        }

        // Eccentric anomaly (intermidiate step to compute mean anomaly)
        let ea = 2.0 * ((ta / 2.0).tan() / ((1.0 + es) / (1.0 - es)).sqrt()).atan();
        // Mean anomaly (it is used because it changes linearly with time,
        // and for that reason is cheap to update)
        let m0 = ea - es * ea.sin();
        // Semi Major Axis
        let a = 1.0 / (2.0 / norm(&r) - sqnorm(&v) / mu);
        Koe::new(a, es, inc, lan, ap, m0, csv.cb.clone())
    }
}

/// # Cartesian State Vectors
/// This structure represents an orbit using a
/// radius vector and a velocity vector.
/// It holds a reference to the central body.
#[derive(Clone)]
pub struct Csv {
    /// Radius vector.
    pub r: Vec3<f64>,
    /// Velocity.
    pub v: Vec3<f64>,
    /// Reference to the central body.
    pub cb: Rc<CB>,
}

impl Csv {
    /// Construct CSV from position and velocity.
    pub fn new(r: glam::DVec3, v: glam::DVec3, cb: Rc<CB>) -> Csv {
        let r = Vec3 {
            x: r.x,
            y: r.y,
            z: r.z,
        };
        let v = Vec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        };
        Csv { r, v, cb }
    }

    /// Construct CSV from position and velocity.
    pub fn nu(r: Vec3<f64>, v: Vec3<f64>, cb: Rc<CB>) -> Csv {
        Csv { r, v, cb }
    }

    /// Construct CSV from KOE.
    pub fn from_koe(koe: Koe) -> Csv {
        // Mean anomaly
        let m0 = koe.m0;
        // Number of iterations for newton_raphson
        let iterations = 10;
        // Eccentric anomaly
        let ea = Csv::newton_raphson(&m0, &koe.e, &iterations);
        // True anomaly
        let ta = 2.0
            * ((1.0 + koe.e).sqrt() * (ea / 2.0).sin())
                .atan2((1.0 - koe.e).sqrt() * (ea / 2.0).cos());
        // Distance to the center of the central body
        let dist = koe.a * (1.0 - koe.e * ea.cos());
        // Radius vector in i, j plane
        let mut r = (koe.cb.i * ta.cos() + koe.cb.j * ta.sin()) * dist;
        // Velocity in i, j plane
        let mut v = (koe.cb.i * (-ea.sin())
            + koe.cb.j * ((1.0 - koe.e.powf(2.0)).sqrt() * ea.cos()))
            * ((koe.cb.mu * koe.a).sqrt() / dist);
        // Radius vector in orbital plane
        r = koe.rot.transform(&r);
        // Velocity in orbital plane
        v = koe.rot.transform(&v);
        Csv::nu(r, v, koe.cb.clone())
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

/// #Central body
/// It holds standard gravitational parameter (product of body's mass and the gravitational constant)
/// and a local coordinate system associated with this body.
pub struct CB {
    /// Standard gravitational parameter.
    pub mu: f64,
    /// A unit vector pointing towards
    /// the intersection of 0th meridian and equator.
    pub i: Vec3<f64>,
    /// j = cross(&k, &i)
    pub j: Vec3<f64>,
    /// A unit vector pointing towards the north pole.
    pub k: Vec3<f64>,
}

impl CB {
    /// Create a new central body asserting that i, j, k vector basis yields an orthonormal right-handed coordinate system.
    pub fn new(mu: f64) -> CB {
        let i = Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let k = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };
        let j = cross(&k, &i);

        // Assert that the basis vectors are orthogonal
        assert!(approx_eq(&dot(&i, &j), &0.0));
        assert!(approx_eq(&dot(&i, &k), &0.0));
        assert!(approx_eq(&dot(&j, &k), &0.0));

        // Assert that the basis vectors all have length 1
        assert!(approx_eq(&norm(&i), &1.0));
        assert!(approx_eq(&norm(&j), &1.0));
        assert!(approx_eq(&norm(&k), &1.0));

        // Assert that the vector basis is right handed
        assert!(approx_eq(&cross(&i, &j), &k));
        CB { mu, i, j, k }
    }
}

fn glamtona(a: glam::DVec3) -> Vec3<f64> {
    Vec3 {
        x: a.x,
        y: a.y,
        z: a.z,
    }
}
