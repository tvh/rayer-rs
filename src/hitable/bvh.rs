use hitable::*;
use pdqselect::select_by;
use std::sync::Arc;
use decorum::Ordered;

pub enum BVH<T: CoordinateBase> {
    Empty,
    Bin {
        left: Arc<BVH<T>>,
        right: Arc<BVH<T>>,
        bbox: BoundingBox<T>,
    },
    Tip {
        hitable: Arc<Hitable<T>>,
        bbox: BoundingBox<T>,
    }
}

impl<T: CoordinateBase> BVH<T> {
    pub fn initialize(items: &[Arc<Hitable<T>>]) -> BVH<T> {
        #[derive(Clone, Copy)]
        enum Axis {
            X, Y, Z
        }
        fn go<T: CoordinateBase>(items: &mut [Arc<Hitable<T>>]) -> BVH<T> {
            match items.len() {
                0 => { return BVH::Empty },
                1 => {
                    let item = items[0].clone();
                    let bbox = item.bbox();
                    return BVH::Tip {
                        hitable: item,
                        bbox: bbox
                    }
                },
                _ => {}
            }
            // Find the "longest" axis
            let mut min_x = T::max_value();
            let mut min_y = T::max_value();
            let mut min_z = T::max_value();
            let mut max_x = T::min_value();
            let mut max_y = T::min_value();
            let mut max_z = T::min_value();
            for item in items.iter() {
                let centroid = item.centroid();
                min_x = T::min(min_x, centroid.x);
                min_y = T::min(min_y, centroid.y);
                min_z = T::min(min_z, centroid.z);
                max_x = T::max(max_x, centroid.x);
                max_y = T::max(max_y, centroid.y);
                max_z = T::max(max_z, centroid.z);
            }
            let width_x = max_x-min_x;
            let width_y = max_y-min_y;
            let width_z = max_z-min_z;
            let mut direction = Axis::X;
            if width_y>width_x {
                direction = Axis::Y;
            }
            if width_z>T::max(width_x, width_y) {
                direction = Axis::Z;
            }
            let split_location = items.len()/2;
            match direction {
                Axis::X => select_by(
                    items, split_location,
                    | a, b | Ordered::from_inner(a.centroid().x).cmp(&Ordered::from_inner(b.centroid().x))
                ),
                Axis::Y => select_by(
                    items, split_location,
                    | a, b | Ordered::from_inner(a.centroid().y).cmp(&Ordered::from_inner(b.centroid().y))
                ),
                Axis::Z => select_by(
                    items, split_location,
                    | a, b | Ordered::from_inner(a.centroid().z).cmp(&Ordered::from_inner(b.centroid().z))
                ),
            };
            let (mut left_items, mut right_items) = items.split_at_mut(split_location);
            let left = go(&mut left_items);
            let right = go(&mut right_items);
            let bbox = left.bbox().merge(right.bbox());
            BVH::Bin{ left: Arc::new(left), right: Arc::new(right), bbox }
        }
        let mut items: Vec<_> = items.iter().map(|x|x.clone()).collect();
        go(items.as_mut_slice())
    }
}

impl<T: CoordinateBase> Hitable<T> for BVH<T> {
    fn bbox(&self) -> BoundingBox<T> {
        match self {
            &BVH::Empty => BoundingBox::<T>::empty(),
            &BVH::Bin { bbox, .. } => bbox,
            &BVH::Tip { bbox, .. } => bbox
        }
    }

    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        match self {
            &BVH::Empty => None,
            &BVH::Tip { ref hitable, bbox } => {
                if bbox.intersects(r, t_min, t_max) {
                    hitable.hit(r, t_min, t_max)
                } else {
                    None
                }
            },
            &BVH::Bin { ref left, ref right, bbox } => {
                if !bbox.intersects(r, t_min, t_max) {
                    return None;
                }
                let mut closest_match = None;
                let mut closest_so_far = t_max;

                match left.hit(r, t_min, closest_so_far) {
                    None => (),
                    Some(hit) => {
                        closest_match = Some(hit);
                        closest_so_far = hit.t;
                    }
                }
                match right.hit(r, t_min, closest_so_far) {
                    None => (),
                    Some(hit) => {
                        closest_match = Some(hit);
                    }
                }
                closest_match
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;
    use palette::*;
    use random::*;
    use num_traits::Float;
    use hitable::sphere::*;

    #[bench]
    fn bench_build_bvh_10000(bench: &mut Bencher) {
        let n = 100000;
        let mut hitables: Vec<Arc<Hitable<f32>>> = black_box(Vec::new());
        let material: Arc<Material<f32>> = Arc::new(Lambertian::new(Rgb::new(0.5, 0.5, 0.5)));
        for _ in 0..10000 {
            let center = rand_in_unit_sphere().to_point();
            let tmp: f32 = rand();
            let radius = tmp/10.0/f32::cbrt(n as f32);
            let sphere = Sphere::new(center, radius, material.clone());
            hitables.push(Arc::new(sphere));
        }
        bench.iter(|| BVH::initialize(hitables.as_slice()) );
    }

    #[bench]
    fn bench_intersect_bvh_10000(bench: &mut Bencher) {
        let n = 100000;
        let mut hitables: Vec<Arc<Hitable<f32>>> = black_box(Vec::new());
        let material: Arc<Material<f32>> = Arc::new(Lambertian::new(Rgb::new(0.5, 0.5, 0.5)));
        for _ in 0..10000 {
            let center = rand_in_unit_sphere().to_point();
            let tmp: f32 = rand();
            let radius = tmp/10.0/f32::cbrt(n as f32);
            let sphere = Sphere::new(center, radius, material.clone());
            hitables.push(Arc::new(sphere));
        }
        let ray = black_box(Ray::new(point3(-3.0, -2.0, -1.0), Vector3D::new(3.0, 2.0, 1.0), 500.0));
        let bvh = BVH::initialize(hitables.as_slice());
        bench.iter(|| bvh.hit(ray, f32::epsilon(), f32::max_value()) );
    }
}
