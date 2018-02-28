use hitable::*;
use pdqselect::select_by;
use std::sync::Arc;
use decorum::Ordered;
use num_traits::Float;

pub struct BVH {
    nodes: Vec<Node>
}

enum Node {
    Bin {
        left_length: usize,
        bbox: AABB,
    },
    Tip {
        hitable: Arc<Hitable>,
        bbox: AABB,
    },
    Empty,
}

impl Node {
    fn bbox(&self) -> AABB {
        match self {
            &Node::Empty => AABB::empty(),
            &Node::Bin{bbox, ..} => bbox,
            &Node::Tip{bbox, ..} => bbox
        }
    }
}

impl BVH {
    pub fn initialize(items: &[Arc<Hitable>]) -> BVH {
        #[derive(Clone, Copy)]
        enum Axis {
            X, Y, Z
        }
        fn go(items: &mut [(Point3D<f32>, Arc<Hitable>)], res: &mut Vec<Node>) -> (AABB, usize) {
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
            let mut min_x = f32::max_value();
            let mut min_y = f32::max_value();
            let mut min_z = f32::max_value();
            let mut max_x = f32::min_value();
            let mut max_y = f32::min_value();
            let mut max_z = f32::min_value();
            for &(centroid, _) in items.iter() {
                min_x = f32::min(min_x, centroid.x);
                min_y = f32::min(min_y, centroid.y);
                min_z = f32::min(min_z, centroid.z);
                max_x = f32::max(max_x, centroid.x);
                max_y = f32::max(max_y, centroid.y);
                max_z = f32::max(max_z, centroid.z);
            }
            let width_x = max_x-min_x;
            let width_y = max_y-min_y;
            let width_z = max_z-min_z;
            let mut direction = Axis::X;
            if width_y>width_x {
                direction = Axis::Y;
            }
            if width_z>f32::max(width_x, width_y) {
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

impl Hitable for BVH {
    fn bbox(&self) -> AABB {
        let &BVH { ref nodes } = self;
        match nodes.as_slice() {
            &[] => AABB::empty(),
            &[Node::Tip {bbox, ..}, ..] => bbox,
            &[Node::Bin {bbox, ..}, ..] => bbox,
            &[Node::Empty, ..] => AABB::empty(),
        }
    }

    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let &BVH { ref nodes } = self;
        fn go(nodes: &[Node], r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
            match nodes {
                &[] => None,
                &[Node::Bin {left_length, ..}, ref left..] => {
                    let right = &left[left_length..];
                    let left_hit = left[0].bbox().intersects(r, t_min, t_max);
                    let right_hit = right[0].bbox().intersects(r, t_min, t_max);

                    match (left_hit, right_hit) {
                        (None, None) => None,
                        (Some(_), None) => go(left, r, t_min, t_max),
                        (None, Some(_)) => go(right, r, t_min, t_max),
                        (Some(left_range), Some(right_range)) => {
                            let mut closest_match = None;
                            let mut closest_so_far = t_max;

                            let (first, (second, second_range)) =
                                if left_range.0<right_range.0 {
                                    (left, (right, right_range))
                                } else {
                                    (right, (left, left_range))
                                };

                            match go(first, r, t_min, t_max) {
                                None => (),
                                Some(hit) => {
                                    closest_match = Some(hit);
                                    closest_so_far = hit.t;
                                }
                            }

                            if closest_so_far<second_range.0 {
                                return closest_match;
                            }

                            match go(second, r, t_min, closest_so_far) {
                                None => (),
                                Some(hit) => {
                                    closest_match = Some(hit);
                                }
                            }

                            closest_match
                        }
                    }
                },
                &[Node::Tip {ref hitable, ..}, ..] => {
                    hitable.hit(r, t_min, t_max)
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
        let mut hitables: Vec<Arc<Hitable>> = black_box(Vec::new());
        let color: Arc<Texture> = Arc::new(Rgb::with_wp(0.5, 0.5, 0.5));
        let material: Arc<Material> = Arc::new(Lambertian::new(&color));
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
        let mut hitables: Vec<Arc<Hitable>> = black_box(Vec::new());
        let color: Arc<Texture> = Arc::new(Rgb::with_wp(0.5, 0.5, 0.5));
        let material: Arc<Material> = Arc::new(Lambertian::new(&color));
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
