use hitable::*;
use pdqselect::select_by;
use std::sync::Arc;
use decorum::Ordered;

pub struct BVH<T: CoordinateBase> {
    nodes: Vec<Node<T>>
}

enum Node<T: CoordinateBase> {
    Bin {
        left_length: usize,
        bbox: AABB<T>,
    },
    Tip {
        hitable: Arc<Hitable<T>>,
        bbox: AABB<T>,
    },
    Empty,
}

impl<T: CoordinateBase> BVH<T> {
    pub fn initialize(items: &[Arc<Hitable<T>>]) -> BVH<T> {
        #[derive(Clone, Copy)]
        enum Axis {
            X, Y, Z
        }
        fn go<T: CoordinateBase>(items: &mut [(Point3D<T>, Arc<Hitable<T>>)], res: &mut Vec<Node<T>>) -> (AABB<T>, usize) {
            match items {
                &mut [] => { return (AABB::empty(), 0); },
                &mut [ref item] => {
                    let item = item.1.clone();
                    let bbox = item.bbox();
                    res.push(Node::Tip {
                        hitable: item,
                        bbox: bbox
                    });
                    return (bbox, 1);
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
            for &(centroid, _) in items.iter() {
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
                    | a, b | Ordered::from_inner(a.0.x).cmp(&Ordered::from_inner(b.0.x))
                ),
                Axis::Y => select_by(
                    items, split_location,
                    | a, b | Ordered::from_inner(a.0.y).cmp(&Ordered::from_inner(b.0.y))
                ),
                Axis::Z => select_by(
                    items, split_location,
                    | a, b | Ordered::from_inner(a.0.z).cmp(&Ordered::from_inner(b.0.z))
                ),
            };
            let (mut left_items, mut right_items) = items.split_at_mut(split_location);
            // Just put a placeholder in to not break the vector
            let current_pos = res.len();
            res.push(Node::Empty);
            let (left_bbox, left_length) = go(&mut left_items, res);
            let (right_bbox, right_length) = go(&mut right_items, res);
            let bbox = left_bbox.merge(right_bbox);
            res[current_pos] = Node::Bin{ left_length, bbox };
            (bbox, 1+left_length+right_length)
        }
        let mut items: Vec<_> = items.iter().map(|x| (x.centroid(), x.clone()) ).collect();
        let mut nodes = Vec::with_capacity(items.len()*2-1);
        go(items.as_mut_slice(), &mut nodes);
        BVH { nodes }
    }
}

impl<T: CoordinateBase> Hitable<T> for BVH<T> {
    fn bbox(&self) -> AABB<T> {
        let &BVH { ref nodes } = self;
        match nodes.as_slice() {
            &[] => AABB::empty(),
            &[Node::Tip {bbox, ..}, ..] => bbox,
            &[Node::Bin {bbox, ..}, ..] => bbox,
            &[Node::Empty, ..] => AABB::empty(),
        }
    }

    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        let &BVH { ref nodes } = self;
        fn go<T: CoordinateBase>(nodes: &[Node<T>], r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
            match nodes {
                &[] => None,
                &[Node::Bin {bbox, left_length}, ref rest..] => {
                    if !bbox.intersects(r, t_min, t_max) {
                        return None;
                    }
                    let mut closest_match = None;
                    let mut closest_so_far = t_max;

                    match go(rest, r, t_min, closest_so_far) {
                        None => (),
                        Some(hit) => {
                            closest_match = Some(hit);
                            closest_so_far = hit.t;
                        }
                    }
                    let right = &rest[left_length..];
                    match go(right, r, t_min, closest_so_far) {
                        None => (),
                        Some(hit) => {
                            closest_match = Some(hit);
                        }
                    }

                    closest_match
                },
                &[Node::Tip {bbox, ref hitable}, ..] => {
                    if bbox.intersects(r, t_min, t_max) {
                        hitable.hit(r, t_min, t_max)
                    } else {
                        None
                    }
                },
                &[Node::Empty, ..] => None,
            }
        }
        go(nodes.as_slice(), r, t_min, t_max)
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
    use pdqselect::select;
    use texture::*;

    fn bench_build(bench: &mut Bencher, n: u64) {
        let mut hitables: Vec<Arc<Hitable<f32>>> = black_box(Vec::new());
        let color: Arc<Texture<f32>> = Arc::new(Rgb::with_wp(0.5, 0.5, 0.5));
        let material: Arc<Material<f32>> = Arc::new(Lambertian::new(&color));
        for _ in 0..n {
            let center = rand_in_unit_sphere().to_point();
            let tmp: f32 = rand();
            let radius = tmp/10.0/f32::cbrt(n as f32);
            let sphere = Sphere::new(center, radius, material.clone());
            hitables.push(Arc::new(sphere));
        }
        bench.iter(|| BVH::initialize(hitables.as_slice()) );
    }

    #[bench]
    fn bench_build_bvh_10000(bench: &mut Bencher) {
        let n = 10000;
        bench_build(bench, n);
    }

    fn bench_intersect_bvh(bench: &mut Bencher, n: u64) {
        let mut hitables: Vec<Arc<Hitable<f32>>> = black_box(Vec::new());
        let color: Arc<Texture<f32>> = Arc::new(Rgb::with_wp(0.5, 0.5, 0.5));
        let material: Arc<Material<f32>> = Arc::new(Lambertian::new(&color));
        for _ in 0..n {
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

    #[bench]
    fn bench_intersect_bvh_10000(bench: &mut Bencher) {
        let n = 10000;
        bench_intersect_bvh(bench, n)
    }

    #[bench]
    fn bench_intersect_bvh_100000(bench: &mut Bencher) {
        let n = 100000;
        bench_intersect_bvh(bench, n)
    }

    #[bench]
    fn bench_intersect_bvh_1000000(bench: &mut Bencher) {
        let n = 1000000;
        bench_intersect_bvh(bench, n)
    }

    #[test]
    fn test_select() {
        let n = 1000;
        let split_location = n/2;
        for _ in 0..100 {
            let mut x: Vec<u64> = Vec::with_capacity(n);
            for _ in 0..n {
                x.push(rand());
            }
            select(&mut x, split_location);
            let pivot = x[split_location];
            for &i in &x[0..split_location] {
                assert!(i<=pivot);
            }
            for &i in &x[split_location..] {
                assert!(i>=pivot);
            }
        }
    }
}
