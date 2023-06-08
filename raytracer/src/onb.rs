use math::vec3::{cross, normalize};

use crate::types::{Real, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local_from_pt(&self, a: Real, b: Real, c: Real) -> Vec3 {
        a * self.u() + b * self.v() + c * self.w()
    }

    pub fn local_from_vec(&self, a: Vec3) -> Vec3 {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }
}

impl std::convert::From<Vec3> for Onb {
    fn from(n: Vec3) -> Self {
        let axis_2 = normalize(n);
        let a = if axis_2.x.abs() > 0.9 {
            math::vec3::consts::unit_y()
        } else {
            math::vec3::consts::unit_x()
        };

        let axis_1 = normalize(cross(axis_2, a));
        let axis_0 = cross(axis_2, axis_1);

        Self {
            axis: [axis_0, axis_1, axis_2],
        }
    }
}

impl std::ops::Index<usize> for Onb {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.axis[index]
    }
}

impl std::ops::IndexMut<usize> for Onb {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.axis[index]
    }
}
