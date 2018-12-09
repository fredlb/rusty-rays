extern crate image;
extern crate rand;
extern crate time;

use std::f32;
mod math;
use math::cross;
use math::dot;
use math::point_at_ray;
use math::random_in_unit_disk;
use math::random_in_unit_sphere;
use math::Ray;
use math::Vector3;
use std::sync::{Arc, Mutex};
use std::thread;
use time::PreciseTime;

const KMIN_T: f32 = 0.001;
const KMAX_T: f32 = 10000000.0;

fn linear_to_srgb(val: f32) -> u8 {
    let mut new = val.max(0.0);
    new = (1.055 * new.powf(0.416666667) - 0.055).max(0.0);
    let new_u8 = (new * 255.9) as u8;
    return new_u8.min(255);
}

#[derive(Debug)]
struct Scatter {
    scattered: Ray,
    attenuation: Vector3,
}

trait Material {
    fn scatter(&self, ray_in: &Ray, hit: &Hit) -> Option<Scatter>;
}

#[derive(Debug, Copy, Clone)]
struct Lambertian {
    albedo: Vector3,
}

#[derive(Debug, Copy, Clone)]
struct Metal {
    albedo: Vector3,
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let target = hit.position + hit.normal + random_in_unit_sphere();
        let scattered = Ray {
            origin: hit.position + 0.001,
            direction: target - hit.position,
        };
        return Some(Scatter {
            attenuation: self.albedo,
            scattered: scattered,
        });
    }
}

fn reflect(v: &Vector3, n: &Vector3) -> Vector3 {
    *v - 2.0 * dot(v, n) * n
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let reflected = reflect(&ray_in.direction.normalize(), &hit.normal);
        let scattered = Ray {
            origin: hit.position,
            direction: reflected,
        };
        if dot(&scattered.direction, &hit.normal) > 0.0 {
            return Some(Scatter {
                attenuation: self.albedo,
                scattered: scattered,
            });
        } else {
            return None;
        }
    }
}

struct Hit<'a> {
    position: Vector3,
    normal: Vector3,
    t: f32,
    material: &'a Box<Material + Send + Sync + 'a>,
}

struct Sphere {
    radius: f32,
    position: Vector3,
    material: Box<Material + Send + Sync>,
}

#[derive(Copy, Clone)]
struct Camera {
    origin: Vector3,
    lower_left_corner: Vector3,
    horizontal: Vector3,
    vertical: Vector3,
    u: Vector3,
    v: Vector3,
    w: Vector3,
    lens_radius: f32,
}

impl Camera {
    fn initialize(
        look_from: Vector3,
        look_at: Vector3,
        v_up: Vector3,
        vfov: f32,
        aspect: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Camera {
        let lens_radius = aperture / 2.0;
        let theta = vfov * f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;

        let w = (look_from - look_at).normalize();
        let u = cross(&v_up, &w).normalize();
        let v = cross(&u, &w);

        return Camera {
            origin: look_from,
            lens_radius: lens_radius,
            w: w,
            u: u,
            v: v,
            lower_left_corner: look_from
                - (u * half_width * focus_dist)
                - (v * half_height * focus_dist)
                - (w * focus_dist),
            horizontal: u * (2.0 * half_width * focus_dist),
            vertical: v * (2.0 * half_height * focus_dist),
        };
    }

    fn make_ray(&self, s: f32, t: f32) -> Ray {
        let rd = random_in_unit_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;
        let direction = self.lower_left_corner + (self.horizontal * s) + (self.vertical * t)
            - self.origin
            - offset;
        return Ray {
            origin: self.origin + offset,
            direction: direction.normalize(),
        };
    }
}

fn ray_sphere_intersection<'a>(
    ray: &Ray,
    sphere: &'a Sphere,
    t_min: f32,
    t_max: f32,
) -> Option<Hit<'a>> {
    let oc = ray.origin - sphere.position;
    let a = dot(&ray.direction, &ray.direction);
    let b = dot(&oc, &ray.direction);
    let c = dot(&oc, &oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - a * c;

    if discriminant > 0.0 {
        let discriminant_sq = discriminant.sqrt();
        let mut t = (-b - discriminant_sq) / a;
        if t < t_max && t > t_min {
            let hit_position = point_at_ray(&ray, t);
            let hit_normal = (hit_position - sphere.position) * (1.0 / sphere.radius);
            return Some(Hit {
                position: hit_position,
                normal: hit_normal.normalize(),
                t: t,
                material: &sphere.material,
            });
        }

        t = (-b + discriminant_sq) / a;
        if t < t_max && t > t_min {
            let hit_position = point_at_ray(&ray, t);
            let hit_normal = (hit_position - sphere.position) * (1.0 / sphere.radius);
            return Some(Hit {
                position: hit_position,
                normal: hit_normal,
                t: t,
                material: &sphere.material,
            });
        }
    }
    return None;
}

fn intersect_scene<'a>(
    ray: &Ray,
    spheres: &'a [Sphere],
    t_min: f32,
    t_max: f32,
) -> Option<Hit<'a>> {
    let mut closest_t = t_max;
    let mut closest_hit = None;
    for i in 0..spheres.len() {
        let result = ray_sphere_intersection(&ray, &spheres[i], t_min, closest_t);
        closest_hit = match result {
            Some(hit) => {
                if hit.t > KMIN_T && hit.t < closest_t {
                    closest_t = hit.t;
                    Some(hit)
                } else {
                    closest_hit
                }
            }
            None => closest_hit,
        };
    }
    closest_hit
}

fn trace(ray: &Ray, spheres: &[Sphere], depth: i32) -> Vector3 {
    if depth > 50 {
        return Vector3::origin();
    }
    let result = intersect_scene(&ray, &spheres, KMIN_T, KMAX_T);
    match result {
        Some(hit) => {
            let scatter = hit.material.scatter(&ray, &hit);
            match scatter {
                Some(scattered) => {
                    return trace(&scattered.scattered, &spheres, depth + 1) * scattered.attenuation;
                }
                None => {
                    return Vector3::origin();
                }
            }
        }
        None => {
            let unit_direction = ray.direction.normalize();
            let t = 0.5 * (unit_direction.y + 1.0);
            return Vector3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vector3::new(0.5, 0.7, 1.0) * t;
        }
    }
}

fn main() {
    let start = PreciseTime::now();
    let sphere_1 = Sphere {
        position: Vector3::new(0.5, 0.01, -1.0),
        radius: 0.5,
        material: Box::new(Lambertian {
            albedo: Vector3::new(1.0, 0.1, 0.1),
        }),
    };
    let sphere_2 = Sphere {
        position: Vector3::new(0.5, -10000.5, -1.0),
        radius: 10000.0,
        material: Box::new(Lambertian {
            albedo: Vector3::new(0.5, 0.5, 0.8),
        }),
    };
    let sphere_3 = Sphere {
        position: Vector3::new(-0.2, -0.295, -1.0),
        radius: 0.2,
        material: Box::new(Metal {
            albedo: Vector3::new(0.5, 0.5, 0.5),
        }),
    };
    let sphere_4 = Sphere {
        position: Vector3::new(-0.8, 0.5, -3.0),
        radius: 1.0,
        material: Box::new(Metal {
            albedo: Vector3::new(0.5, 0.5, 0.5),
        }),
    };
    let spheres = Arc::new(vec![sphere_1, sphere_2, sphere_3, sphere_4]);

    let look_from = Vector3::new(0.0, 0.0, 3.0);
    let look_at = Vector3::new(0.0, 0.0, -1.0);
    let dist_to_focus = 3.0;
    let aperture = 0.00;

    let screen_height = 400;
    let screen_width = 600;
    let spp = 128;

    let camera = Camera::initialize(
        look_from,
        look_at,
        Vector3::new(0.0, 1.0, 0.0),
        30.0,
        screen_width as f32 / screen_height as f32,
        aperture,
        dist_to_focus,
    );
    let mut handlers = vec![];

    let imgbuf = Arc::new(Mutex::new(image::RgbImage::new(
        screen_width,
        screen_height,
    )));
    let threads = 4;
    for t in 0..threads {
        let mut i = t;
        let local_scene = spheres.clone();
        let imagebuf = Arc::clone(&imgbuf);
        let handle = thread::spawn(move || {
            while i < screen_height {
                for j in 0..screen_width {
                    let mut color = Vector3::origin();
                    for _ in 0..spp {
                        let u: f32 = (j as f32 + rand::random::<f32>()) / screen_width as f32;
                        let v: f32 = (i as f32 + rand::random::<f32>()) / screen_height as f32;
                        let ray = &camera.clone().make_ray(u, v);
                        color = color + trace(&ray, &local_scene, 0);
                    }
                    let mut image = imagebuf.lock().unwrap();
                    let pixel = image::Rgb([
                        linear_to_srgb(color.x / spp as f32),
                        linear_to_srgb(color.y / spp as f32),
                        linear_to_srgb(color.z / spp as f32),
                    ]);
                    image.put_pixel(j, i, pixel);
                }
                i += threads;
            }
        });
        handlers.push(handle);
    }

    for handle in handlers {
        handle.join().unwrap();
    }

    imgbuf.lock().unwrap().save("output.png").unwrap();
    let end = PreciseTime::now();

    println!(
        "Done! It took {} seconds to complete with {} threads",
        start.to(end),
        threads
    );
}
