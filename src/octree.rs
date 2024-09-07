use std::ops::{Index, IndexMut};

pub(crate) type Point = [f64; 3];

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default, Hash)]
pub(crate) struct OctantId(pub usize);

#[derive(Clone, Debug, Default)]
pub(crate) struct Octant {
    // Tree data
    pub parent: Option<OctantId>,
    pub children: Vec<OctantId>,

    // Node Data
    pub centre: Point,
    pub extent: f64,
    pub ipoints: Vec<usize>,
    pub ranking: usize,
}

impl Octant {
    pub fn new(extent: f64) -> Self {
        // Ensure a positive extent
        assert!(extent.is_sign_positive());

        Self {
            extent,
            ..Default::default()
        }
    }

    pub fn from_points(points: &[Point]) -> Self {
        use vecfx::*;

        // Find boundary from points
        let x_arr: Vec<_> = points.iter().map(|[x, _, _]| *x).collect();
        let y_arr: Vec<_> = points.iter().map(|[_, y, _]| *y).collect();
        let z_arr: Vec<_> = points.iter().map(|[_, _, z]| *z).collect();

        // Find min and max for each component
        let (x_min, y_min, z_min) = (x_arr.min(), y_arr.min(), z_arr.min());
        let (x_max, y_max, z_max) = (x_arr.max(), y_arr.max(), z_arr.max());

        let (wx, wy, wz) = (x_max - x_min, y_max - y_min, z_max - z_min);
        let width = [wx, wy, wz].max();

        // Construct the root node which contains all the points
        let mut root = Octant::new(0.5 * width);
        root.centre = [
            (x_max + x_min) / 2.,
            (y_max + y_min) / 2.,
            (z_max + z_min) / 2.,
        ];
        root.ipoints = (0..points.len()).collect();

        root
    }

    pub fn neighbouring(&self, other: &Octant) -> bool {
        for i in 0..3 {
            let v = (other.centre[i] - self.centre[i]).abs() - (other.extent + self.extent);

            if v > 0.001 {
                return false;
            }
        }

        true
    }
}

#[derive(Debug)]
pub(crate) struct Octree {
    pub points: Vec<Point>,

    octants: Vec<Octant>,
    root: OctantId,
}

impl Octree {
    pub fn new(points: impl IntoIterator<Item = Point>) -> Self {
        let points: Vec<_> = points.into_iter().collect();
        let octant = Octant::from_points(&points);
        let octants = vec![octant];

        let root = OctantId(0);

        Self {
            points,
            octants,
            root,
        }
    }

    fn root(&self) -> OctantId {
        OctantId(0)
    }

    pub fn count(&self) -> usize {
        self.octants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.octants.is_empty()
    }

    fn new_node(&mut self, octant: Octant) -> OctantId {
        let next_i = self.octants.len();

        self.octants.push(octant);

        OctantId(next_i)
    }

    fn append_child(&mut self, parent_node: OctantId, mut octant: Octant) -> OctantId {
        octant.parent = Some(parent_node);
        octant.ranking = self[parent_node].children.len();
        let n = self.new_node(octant);

        // Get parent octant and update children attributes
        let parent_octant = &mut self[parent_node];
        parent_octant.children.push(n);

        n
    }

    fn split_octant(&mut self, parent_node: OctantId) {
        let child_octants = Self::octree_create_child_octants(&self[parent_node], &self.points);

        for octant in child_octants {
            self.append_child(parent_node, octant);
        }
    }

    pub fn build(&mut self, bucket_size: usize) {
        debug_assert!(bucket_size > 0, "invalid bucket_size: {}!", bucket_size);

        let root = self.root();
        let npoints = self.points.len();
        if npoints > bucket_size {
            let mut need_split = vec![root];
            for depth in 0.. {
                // 1. split into child octants
                let mut remained = vec![];
                for &parent_node in need_split.iter() {
                    self.split_octant(parent_node);
                    for &child_node in &self[parent_node].children {
                        let octant = &self[child_node];
                        let n = octant.ipoints.len();
                        if n > bucket_size {
                            remained.push(child_node);
                        }
                    }
                }

                // 2. drill down to process child octants
                need_split.clear();
                need_split.extend(remained);

                // 3. loop control
                if need_split.is_empty() {
                    println!("octree built after {:?} cycles.", depth);
                    break;
                }
            }

            // cache octants
            // create mapping of point => octant
            // for (i, ref octant) in self.octants.iter().enumerate() {
            //     for &j in octant.ipoints.iter() {
            //         self.mapping_octants.insert(j, i);
            //     }
            // }
        }
    }

    fn octree_create_child_octants(octant: &Octant, points: &[Point]) -> Vec<Octant> {
        let extent = octant.extent as f64 / 2.;

        // Initialise 8 child octants
        // 1. Update centre
        let mut child_octants: Vec<_> = (0..8)
            .map(|i| {
                let mut o = Octant::new(extent);
                let factors = Self::get_octant_cell_factor(i);

                // j = 0, 1, 2 => x, y, z
                for j in 0..3 {
                    o.centre[j] += extent * factors[j] + octant.centre[j]
                }

                o
            })
            .collect();

        // 2. Update point indices
        if octant.ipoints.len() > 1 {
            let (x0, y0, z0) = (octant.centre[0], octant.centre[1], octant.centre[2]);

            // Scan xyz
            for &i in octant.ipoints.iter() {
                let p = points[i];
                let (x, y, z) = (p[0] - x0, p[1] - y0, p[2] - z0);
                let index = Self::get_octant_cell_index(x, y, z);

                child_octants[index].ipoints.push(i);
            }
        }

        child_octants
    }

    // zyx: +++ => 0
    // zyx: ++- => 1
    // zyx: --- => 7
    // morton encode
    #[inline]
    fn get_octant_cell_index(x: f64, y: f64, z: f64) -> usize {
        // create lookup table, which could be faster
        match (
            z.is_sign_positive(),
            y.is_sign_positive(),
            x.is_sign_positive(),
        ) {
            (true, true, true) => 0,
            (true, true, false) => 1,
            (true, false, true) => 2,
            (true, false, false) => 3,
            (false, true, true) => 4,
            (false, true, false) => 5,
            (false, false, true) => 6,
            (false, false, false) => 7,
        }

        // another way: using bit shift
        // let bits = [z.is_sign_negative(), y.is_sign_negative(), x.is_sign_negative()];
        // bits.iter().fold(0, |acc, &b| acc*2 + b as usize)
    }

    // useful for calculate center of child octant
    // morton decode
    #[inline]
    fn get_octant_cell_factor(index: usize) -> Point {
        debug_assert!(index < 8);
        [
            match (index & 0b001) == 0 {
                true => 1.0,
                false => -1.0,
            },
            match ((index & 0b010) >> 1) == 0 {
                true => 1.0,
                false => -1.0,
            },
            match ((index & 0b100) >> 2) == 0 {
                true => 1.0,
                false => -1.0,
            },
        ]
    }
}

impl Index<OctantId> for Octree {
    type Output = Octant;

    fn index(&self, node: OctantId) -> &Octant {
        &self.octants[node.0]
    }
}

impl IndexMut<OctantId> for Octree {
    fn index_mut(&mut self, node: OctantId) -> &mut Octant {
        &mut self.octants[node.0]
    }
}
