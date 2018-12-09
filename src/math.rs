extern crate rand;

use std::f32;
use std::ops;

#[derive(Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    pub fn origin() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn length(self) -> f32 {
        return dot(&self, &self).sqrt();
    }

    pub fn normalize(self) -> Vector3 {
        let k = 1.0 / self.length();
        return self * k;
    }
}

impl ops::Add for Vector3 {
    type Output = Vector3;

    fn add(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<'a, 'b> ops::Sub<&'b Vector3> for &'a Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.x,
            z: self.z * rhs.x,
        }
    }
}
impl ops::Add<f32> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

pub fn dot(v1: &Vector3, v2: &Vector3) -> f32 {
    return v1.x * v2.x + v1.y * v2.y + v1.z * v2.z;
}

pub fn random_in_unit_sphere() -> Vector3 {
    let mut p = Vector3::new(rand::random(), rand::random(), rand::random()) * 2.0
        - Vector3::new(1.0, 1.0, 1.0);
    while dot(&p, &p) >= 1.0 {
        p = Vector3::new(rand::random(), rand::random(), rand::random()) * 2.0
            - Vector3::new(1.0, 1.0, 1.0);
    }
    return p;
}

pub fn random_in_unit_disk() -> Vector3 {
    let mut p =
        Vector3::new(rand::random(), rand::random(), 0.0) * 2.0 - Vector3::new(1.0, 1.0, 0.0);
    while dot(&p, &p) >= 1.0 {
        p = Vector3::new(rand::random(), rand::random(), 0.0) * 2.0 - Vector3::new(1.0, 1.0, 0.0);
    }
    return p;
}

pub fn cross(v1: &Vector3, v2: &Vector3) -> Vector3 {
    return Vector3::new(
        v1.y * v2.z - v1.z * v2.y,
        -(v1.x * v2.z - v1.z * v2.x),
        v1.x * v2.y - v1.y * v2.x,
    );
}

pub fn point_at_ray(ray: &Ray, t: f32) -> Vector3 {
    return ray.origin + ray.direction * t;
}
