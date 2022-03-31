use euclid::*;
use std::sync::Arc;
use std::path::Path;
use std::io::Error;
use obj::{SimplePolygon, Obj};

use hitable::*;
use hitable::bvh::BVH;
use texture::Texture;

#[derive(Debug, Clone)]
pub struct Triangle {
    vert: (Point3D<f32, UnknownUnit>, Point3D<f32, UnknownUnit>, Point3D<f32, UnknownUnit>),
    normal: (Vector3D<f32, UnknownUnit>, Vector3D<f32, UnknownUnit>, Vector3D<f32, UnknownUnit>),
    uv: (Vector2D<f32, UnknownUnit>, Vector2D<f32, UnknownUnit>, Vector2D<f32, UnknownUnit>),
    texture: Arc<dyn Texture>,
}

impl Triangle {
    pub fn new(
        vert: (Point3D<f32, UnknownUnit>, Point3D<f32, UnknownUnit>, Point3D<f32, UnknownUnit>),
        normal: (Vector3D<f32, UnknownUnit>, Vector3D<f32, UnknownUnit>, Vector3D<f32, UnknownUnit>),
        uv: (Vector2D<f32, UnknownUnit>, Vector2D<f32, UnknownUnit>, Vector2D<f32, UnknownUnit>),
        texture: Arc<dyn Texture>,
    ) -> Triangle {
        Triangle {
            vert,
            normal,
            uv,
            texture,
        }
    }
}

pub fn polygon(
    data: &[(Point3D<f32, UnknownUnit>, Vector3D<f32, UnknownUnit>, Vector2D<f32, UnknownUnit>)],
    texture: Arc<dyn Texture>,
) -> Vec<Triangle> {
    let mut res = Vec::with_capacity(data.len()-2);
    match data {
        &[] => return res,
        &[(p0, n0, t0), ref rest @ ..] => {
            for (&(p1, n1, t1), &(p2, n2, t2)) in rest.iter().zip(rest[1..].iter()) {
                res.push(Triangle::new(
                    (p0, p1, p2),
                    (n0, n1, n2),
                    (t0, t1, t2),
                    texture.clone()
                ));
            };
            return res;
        }
    }
}

impl Hitable for Triangle {
    fn bbox(&self) -> AABB {
        let mut low = self.vert.0;
        let mut high = self.vert.0;
        for obj in [self.vert.1, self.vert.2].iter() {
            low = point3(
                f32::min(low.x, obj.x),
                f32::min(low.y, obj.y),
                f32::min(low.z, obj.z),
            );
            high = point3(
                f32::max(high.x, obj.x),
                f32::max(high.y, obj.y),
                f32::max(high.z, obj.z),
            );
        }
        AABB { bounds: [low, high] }
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // find vectors for two edges sharing vert0
        let edge1 = self.vert.1 - self.vert.0;
        let edge2 = self.vert.2 - self.vert.0;
        // begin calculating determinant also used to calculate U parameter
        let pvec = r.direction.cross(edge2);
        // if determinant is near zero ray lies in plane of triangle
        let det = edge1.dot(pvec);
        if !det.is_normal() {
            return None;
        }
        let inv_det = det.recip();
        // calculate distance from vert0 to ray origin
        let tvec = r.origin - self.vert.0;
        // calculate U parameter and test bounds
        let u = tvec.dot(pvec) * inv_det;
        if u<0.0 || u>1.0 {
            return None;
        }
        // prepare to test V parameter
        let qvec = tvec.cross(edge1);
        // calculate V parameter and test bounds
        let v = r.direction.dot(qvec) * inv_det;
        if v<0.0 || v>1.0 {
            return None;
        }
        // calculate t, ray intersects triangle
        let t = edge2.dot(qvec) * inv_det;
        if t<=t_min || t>=t_max {
            return None;
        }
        let w = 1.0 - u - v;
        if w<0.0 || w>1.0 {
            return None;
        }
        let normal = (self.normal.0*v + self.normal.1*u + self.normal.2*w).normalize();
        let p = r.point_at_parameter(t);
        let uv = self.uv.0*v + self.uv.1*u + self.uv.2*w;
        Some(HitRecord{p, t, normal, texture: self.texture.as_ref(), uv})
    }
}

/// Construct a polygon from a number of points.
/// All points should be on the same plane.
/// The texture coordinates will always be mapped to (0,0)
pub fn uniform_polygon(
    data: &[Point3D<f32, UnknownUnit>],
    normal: Vector3D<f32, UnknownUnit>,
    material: Arc<dyn Texture>,
) -> Vec<Triangle> {
    let mut args = Vec::with_capacity(data.len());
    for &p in data.iter() {
        args.push((p, normal, vec2(0.0, 0.0)));
    }
    polygon(args.as_slice(), material.into())
}

#[derive(Debug, Clone)]
pub struct Mesh {
    data: Arc<BVH<Triangle>>
}

impl Mesh {
    /// Load an obj file from disk.
    /// It currently ignores the material stored in the file,
    /// but loads the texture coordinates correctly.
    /// If there are no texture coordinates, they will all be mapped to (0,0).
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rayer;
    /// # extern crate palette;
    /// # extern crate euclid;
    /// # use euclid::point3;
    /// # use rayer::texture::Texture;
    /// # use rayer::hitable::*;
    /// # use rayer::hitable::triangle::*;
    /// # use std::sync::Arc;
    /// # use palette::*;
    /// # use rayer::material::*;
    /// # use std::path::Path;
    /// #
    /// # let texture: Arc<Texture> = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
    /// let bunny = Mesh::from_obj(Path::new("data/bunny.obj"), texture).unwrap();
    /// assert_ne!(AABB::empty(), bunny.bbox());
    /// ```
    pub fn from_obj(
        path: &Path,
        texture: Arc<dyn Texture>
    ) -> Result<Mesh, Error> {
        let obj: Obj<'_, SimplePolygon> = Obj::load(path)?;
        let mut triangles: Vec<Triangle> = Vec::new();
        let get_normal = |i| Vector3D::from(obj.normal[i]);

        for o in obj.objects.iter() {
            for g in o.groups.iter() {
                for p in g.polys.iter() {
                    let p0 = p[0];
                    let vert0 = obj.position[p0.0].into();
                    for (p1, p2) in p[1..p.len()-1].iter().zip(p[2..].iter()) {
                        let vert1 = obj.position[p1.0].into();
                        let vert2 = obj.position[p2.0].into();

                        let v: Vector3D<f32, UnknownUnit> = vert1-vert0;
                        let w: Vector3D<f32, UnknownUnit> = vert2-vert0;
                        let default_normal = v.cross(w);

                        let normal0 = p0.2.map_or(default_normal, &get_normal);
                        let normal1 = p1.2.map_or(default_normal, &get_normal);
                        let normal2 = p2.2.map_or(default_normal, &get_normal);

                        let uv0 = p0.1.map_or(vec2(0.0, 0.0), |i| obj.texture[i].into());
                        let uv1 = p1.1.map_or(vec2(0.0, 0.0), |i| obj.texture[i].into());
                        let uv2 = p2.1.map_or(vec2(0.0, 0.0), |i| obj.texture[i].into());

                        triangles.push(Triangle::new(
                            (vert0, vert1, vert2),
                            (normal0, normal1, normal2),
                            (uv0, uv1, uv2),
                            texture.clone(),
                        ));
                    }
                }
            }
        }

        Ok(Mesh{ data: Arc::new(BVH::initialize(triangles)) })
    }
}

impl Hitable for Mesh {
    fn bbox(&self) -> AABB {
        self.data.bbox()
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.data.hit(r, t_min, t_max)
    }
}

/// Build an axis aligned cuboid.
/// For now all texture coordinated will be mapped to (0, 0)
pub fn axis_aligned_cuboid(
    l: Point3D<f32, UnknownUnit>,
    h: Point3D<f32, UnknownUnit>,
    texture: Arc<dyn Texture>
) -> Mesh {
    let mut triangles = Vec::with_capacity(12);
    triangles.extend_from_slice(uniform_polygon(
        &[l, point3(l.x, h.y, l.z), point3(l.x, h.y, h.z), point3(l.x, l.y, h.z)],
        vec3(-1.0, 0.0, 0.0),
        texture.clone()
    ).as_slice());
    triangles.extend_from_slice(uniform_polygon(
        &[l, point3(h.x, l.y, l.z), point3(h.x, l.y, h.z), point3(l.x, l.y, h.z)],
        vec3(0.0, -1.0, 0.0),
        texture.clone()
    ).as_slice());
    triangles.extend_from_slice(uniform_polygon(
        &[l, point3(h.x, l.y, l.z), point3(h.x, h.y, l.z), point3(l.x, h.y, l.z)],
        vec3(0.0, 0.0, -1.0),
        texture.clone()
    ).as_slice());
    triangles.extend_from_slice(uniform_polygon(
        &[h, point3(h.x, h.y, l.z), point3(h.x, l.y, l.z), point3(h.x, l.y, h.z)],
        vec3(1.0, 0.0, 0.0),
        texture.clone()
    ).as_slice());
    triangles.extend_from_slice(uniform_polygon(
        &[h, point3(h.x, h.y, l.z), point3(l.x, h.y, l.z), point3(l.x, h.y, h.z)],
        vec3(0.0, 1.0, 0.0),
        texture.clone()
    ).as_slice());
    triangles.extend_from_slice(uniform_polygon(
        &[h, point3(h.x, l.y, h.z), point3(l.x, l.y, h.z), point3(l.x, h.y, h.z)],
        vec3(0.0, 0.0, 1.0),
        texture.clone()
    ).as_slice());

    Mesh { data: Arc::new(BVH::initialize(triangles)) }
}
