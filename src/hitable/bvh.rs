use hitable::*;
use pdqselect::select_by;
use decorum::Ordered;
use std::ptr;

#[derive(Debug)]
pub struct BVH<H: Hitable> {
    nodes: Vec<Node>,
    items: Vec<H>,
}

#[derive(Debug)]
struct Node {
    bbox: AABB,
    next: Next,
}

#[derive(Debug)]
enum Next {
    Bin { left_length: usize },
    Tip { hitable: usize },
}

impl<H: Hitable> BVH<H> {
    pub fn initialize(items: Vec<H>) -> BVH<H> {
        #[derive(Clone, Copy)]
        enum Axis {
            X, Y, Z
        }
        fn go(items: &mut [(Point3D<f32>, usize, AABB)], res: &mut Vec<Node>) -> (AABB, usize) {
            match items {
                &mut [] => { return (AABB::empty(), 0); },
                &mut [ref item] => {
                    let bbox = item.2;
                    res.push(Node {
                        next: Next::Tip { hitable: item.1},
                        bbox,
                    });
                    return (bbox, 1);
                },
                _ => {}
            }
            // Find the "longest" axis
            let mut min_x = items[0].0.x;
            let mut min_y = items[0].0.y;
            let mut min_z = items[0].0.z;
            let mut max_x = items[0].0.x;
            let mut max_y = items[0].0.y;
            let mut max_z = items[0].0.z;
            for &(centroid, _, _) in items[1..].iter() {
                if min_x>centroid.x { min_x=centroid.x };
                if min_y>centroid.y { min_y=centroid.y };
                if min_z>centroid.z { min_z=centroid.z };
                if max_x<centroid.x { max_x=centroid.x };
                if max_y<centroid.y { max_y=centroid.y };
                if max_z<centroid.z { max_z=centroid.z };
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
            let current_pos = res.len();
            // This spot will be filled later
            unsafe { res.set_len(current_pos+1) };
            let (left_bbox, left_length) = go(&mut left_items, res);
            let (right_bbox, right_length) = go(&mut right_items, res);
            let bbox = left_bbox.merge(right_bbox);
            unsafe {
                ptr::write(
                    res.as_mut_ptr().offset(current_pos as isize),
                    Node {bbox, next: Next::Bin{ left_length } }
                );
            };
            (bbox, 1+left_length+right_length)
        }
        let mut item_stats: Vec<(Point3D<f32>, usize, AABB)> = items.iter().enumerate().map(|(i, x)| (x.centroid(), i, x.bbox())).collect();
        let mut nodes: Vec<Node> = Vec::with_capacity(items.len()*2-1);
        go(item_stats.as_mut_slice(), &mut nodes);
        BVH { nodes, items }
    }
}

impl<H: Hitable> Hitable for BVH<H> {
    fn bbox(&self) -> AABB {
        let &BVH { ref nodes, .. } = self;
        match nodes.as_slice() {
            &[] => AABB::empty(),
            &[Node {bbox, ..}, ..] => bbox,
        }
    }

    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let &BVH { ref nodes, ref items } = self;
        fn go<'a, H: Hitable>(items: &'a[H], nodes: &[Node], r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'a>> {
            match nodes {
                &[] => None,
                &[Node { next: Next::Bin{left_length}, ..}, ref left..] => {
                    let right = &left[left_length..];
                    let (left_hit, right_hit) = left[0].bbox.intersects_2(&right[0].bbox, r, t_min, t_max);

                    match (left_hit, right_hit) {
                        (None, None) => None,
                        (Some(_), None) => go(items, left, r, t_min, t_max),
                        (None, Some(_)) => go(items, right, r, t_min, t_max),
                        (Some(left_range), Some(right_range)) => {
                            let mut closest_match = None;
                            let mut closest_so_far = t_max;

                            let (first, (second, second_range)) =
                                if left_range.0<right_range.0 {
                                    (left, (right, right_range))
                                } else {
                                    (right, (left, left_range))
                                };

                            match go(items, first, r, t_min, t_max) {
                                None => (),
                                Some(hit) => {
                                    closest_so_far = hit.t;
                                    closest_match = Some(hit);
                                }
                            }

                            if closest_so_far<second_range.0 {
                                return closest_match;
                            }

                            match go(items, second, r, t_min, closest_so_far) {
                                None => (),
                                Some(hit) => {
                                    closest_match = Some(hit);
                                }
                            }

                            closest_match
                        }
                    }
                },
                &[Node {next: Next::Tip{hitable}, ..}, ..] => {
                    items[hitable].hit(r, t_min, t_max)
                },
            }
        }
        go(items.as_slice(), nodes.as_slice(), r, t_min, t_max)
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
    use material::*;
    use std::sync::Arc;

    fn bench_build(bench: &mut Bencher, n: u64) {
        let mut hitables: Vec<Arc<Hitable>> = black_box(Vec::new());
        let texture: Arc<Texture> = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
        for _ in 0..n {
            let center = rand_in_unit_sphere().to_point();
            let tmp: f32 = rand();
            let radius = tmp/10.0/f32::cbrt(n as f32);
            let sphere = Sphere::new(center, radius, texture.clone());
            hitables.push(Arc::new(sphere));
        }
        bench.iter(|| {
            black_box(BVH::initialize(hitables.clone()))
        });
    }

    #[bench]
    fn bench_build_bvh_10000(bench: &mut Bencher) {
        let n = 10000;
        bench_build(bench, n);
    }

    fn bench_intersect_bvh(bench: &mut Bencher, n: u64) {
        let mut hitables: Vec<Sphere> = black_box(Vec::new());
        let texture: Arc<Texture> = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
        for _ in 0..n {
            let center = rand_in_unit_sphere().to_point();
            let tmp: f32 = rand();
            let radius = tmp/10.0/f32::cbrt(n as f32);
            let sphere = Sphere::new(center, radius, texture.clone());
            hitables.push(sphere);
        }
        let ray = black_box(Ray::new(point3(-3.0, -2.0, -1.0), Vector3D::new(3.0, 2.0, 1.0), 500.0, 0.0));
        let bvh = BVH::initialize(hitables);
        bench.iter(|| black_box(bvh.hit(ray, f32::epsilon(), f32::max_value())) );
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
