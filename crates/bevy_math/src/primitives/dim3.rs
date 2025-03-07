use super::{InvalidDirectionError, Primitive3d};
use crate::Vec3;

/// A normalized vector pointing in a direction in 3D space
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Direction3d(Vec3);

impl Direction3d {
    /// A unit vector pointing along the positive X axis.
    pub const X: Self = Self(Vec3::X);
    /// A unit vector pointing along the positive Y axis.
    pub const Y: Self = Self(Vec3::Y);
    /// A unit vector pointing along the positive Z axis.
    pub const Z: Self = Self(Vec3::Z);
    /// A unit vector pointing along the negative X axis.
    pub const NEG_X: Self = Self(Vec3::NEG_X);
    /// A unit vector pointing along the negative Y axis.
    pub const NEG_Y: Self = Self(Vec3::NEG_Y);
    /// A unit vector pointing along the negative Z axis.
    pub const NEG_Z: Self = Self(Vec3::NEG_Z);

    /// Create a direction from a finite, nonzero [`Vec3`].
    ///
    /// Returns [`Err(InvalidDirectionError)`](InvalidDirectionError) if the length
    /// of the given vector is zero (or very close to zero), infinite, or `NaN`.
    pub fn new(value: Vec3) -> Result<Self, InvalidDirectionError> {
        Self::new_and_length(value).map(|(dir, _)| dir)
    }

    /// Create a [`Direction3d`] from a [`Vec3`] that is already normalized.
    ///
    /// # Warning
    ///
    /// `value` must be normalized, i.e it's length must be `1.0`.
    pub fn new_unchecked(value: Vec3) -> Self {
        debug_assert!(value.is_normalized());

        Self(value)
    }

    /// Create a direction from a finite, nonzero [`Vec3`], also returning its original length.
    ///
    /// Returns [`Err(InvalidDirectionError)`](InvalidDirectionError) if the length
    /// of the given vector is zero (or very close to zero), infinite, or `NaN`.
    pub fn new_and_length(value: Vec3) -> Result<(Self, f32), InvalidDirectionError> {
        let length = value.length();
        let direction = (length.is_finite() && length > 0.0).then_some(value / length);

        direction
            .map(|dir| (Self(dir), length))
            .ok_or(InvalidDirectionError::from_length(length))
    }

    /// Create a direction from its `x`, `y`, and `z` components.
    ///
    /// Returns [`Err(InvalidDirectionError)`](InvalidDirectionError) if the length
    /// of the vector formed by the components is zero (or very close to zero), infinite, or `NaN`.
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Result<Self, InvalidDirectionError> {
        Self::new(Vec3::new(x, y, z))
    }
}

impl TryFrom<Vec3> for Direction3d {
    type Error = InvalidDirectionError;

    fn try_from(value: Vec3) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::ops::Deref for Direction3d {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Neg for Direction3d {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

/// A sphere primitive
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    /// The radius of the sphere
    pub radius: f32,
}
impl Primitive3d for Sphere {}

impl Sphere {
    /// Create a new [`Sphere`] from a `radius`
    #[inline(always)]
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }

    /// Finds the point on the sphere that is closest to the given `point`.
    ///
    /// If the point is outside the sphere, the returned point will be on the surface of the sphere.
    /// Otherwise, it will be inside the sphere and returned as is.
    #[inline(always)]
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        let distance_squared = point.length_squared();

        if distance_squared <= self.radius.powi(2) {
            // The point is inside the sphere.
            point
        } else {
            // The point is outside the sphere.
            // Find the closest point on the surface of the sphere.
            let dir_to_point = point / distance_squared.sqrt();
            self.radius * dir_to_point
        }
    }
}

/// An unbounded plane in 3D space. It forms a separating surface through the origin,
/// stretching infinitely far
#[derive(Clone, Copy, Debug)]
pub struct Plane3d {
    /// The normal of the plane. The plane will be placed perpendicular to this direction
    pub normal: Direction3d,
}
impl Primitive3d for Plane3d {}

impl Plane3d {
    /// Create a new `Plane3d` from a normal
    ///
    /// # Panics
    ///
    /// Panics if the given `normal` is zero (or very close to zero), or non-finite.
    #[inline]
    pub fn new(normal: Vec3) -> Self {
        Self {
            normal: Direction3d::new(normal).expect("normal must be nonzero and finite"),
        }
    }
}

/// An infinite line along a direction in 3D space.
///
/// For a finite line: [`Segment3d`]
#[derive(Clone, Copy, Debug)]
pub struct Line3d {
    /// The direction of the line
    pub direction: Direction3d,
}
impl Primitive3d for Line3d {}

/// A segment of a line along a direction in 3D space.
#[doc(alias = "LineSegment3d")]
#[derive(Clone, Debug)]
pub struct Segment3d {
    /// The direction of the line
    pub direction: Direction3d,
    /// Half the length of the line segment. The segment extends by this amount in both
    /// the given direction and its opposite direction
    pub half_length: f32,
}
impl Primitive3d for Segment3d {}

impl Segment3d {
    /// Create a line segment from a direction and full length of the segment
    pub fn new(direction: Direction3d, length: f32) -> Self {
        Self {
            direction,
            half_length: length / 2.,
        }
    }

    /// Get a line segment and translation from two points at each end of a line segment
    ///
    /// Panics if point1 == point2
    pub fn from_points(point1: Vec3, point2: Vec3) -> (Self, Vec3) {
        let diff = point2 - point1;
        let length = diff.length();

        (
            // We are dividing by the length here, so the vector is normalized.
            Self::new(Direction3d::new_unchecked(diff / length), length),
            (point1 + point2) / 2.,
        )
    }

    /// Get the position of the first point on the line segment
    pub fn point1(&self) -> Vec3 {
        *self.direction * -self.half_length
    }

    /// Get the position of the second point on the line segment
    pub fn point2(&self) -> Vec3 {
        *self.direction * self.half_length
    }
}

/// A series of connected line segments in 3D space.
///
/// For a version without generics: [`BoxedPolyline3d`]
#[derive(Clone, Debug)]
pub struct Polyline3d<const N: usize> {
    /// The vertices of the polyline
    pub vertices: [Vec3; N],
}
impl<const N: usize> Primitive3d for Polyline3d<N> {}

impl<const N: usize> FromIterator<Vec3> for Polyline3d<N> {
    fn from_iter<I: IntoIterator<Item = Vec3>>(iter: I) -> Self {
        let mut vertices: [Vec3; N] = [Vec3::ZERO; N];

        for (index, i) in iter.into_iter().take(N).enumerate() {
            vertices[index] = i;
        }
        Self { vertices }
    }
}

impl<const N: usize> Polyline3d<N> {
    /// Create a new `Polyline3d` from its vertices
    pub fn new(vertices: impl IntoIterator<Item = Vec3>) -> Self {
        Self::from_iter(vertices)
    }
}

/// A series of connected line segments in 3D space, allocated on the heap
/// in a `Box<[Vec3]>`.
///
/// For a version without alloc: [`Polyline3d`]
#[derive(Clone, Debug)]
pub struct BoxedPolyline3d {
    /// The vertices of the polyline
    pub vertices: Box<[Vec3]>,
}
impl Primitive3d for BoxedPolyline3d {}

impl FromIterator<Vec3> for BoxedPolyline3d {
    fn from_iter<I: IntoIterator<Item = Vec3>>(iter: I) -> Self {
        let vertices: Vec<Vec3> = iter.into_iter().collect();
        Self {
            vertices: vertices.into_boxed_slice(),
        }
    }
}

impl BoxedPolyline3d {
    /// Create a new `BoxedPolyline3d` from its vertices
    pub fn new(vertices: impl IntoIterator<Item = Vec3>) -> Self {
        Self::from_iter(vertices)
    }
}

/// A cuboid primitive, more commonly known as a box.
#[derive(Clone, Copy, Debug)]
pub struct Cuboid {
    /// Half of the width, height and depth of the cuboid
    pub half_size: Vec3,
}
impl Primitive3d for Cuboid {}

impl Cuboid {
    /// Create a cuboid from a full x, y, and z length
    pub fn new(x_length: f32, y_length: f32, z_length: f32) -> Self {
        Self::from_size(Vec3::new(x_length, y_length, z_length))
    }

    /// Create a cuboid from a given full size
    pub fn from_size(size: Vec3) -> Self {
        Self {
            half_size: size / 2.,
        }
    }

    /// Finds the point on the cuboid that is closest to the given `point`.
    ///
    /// If the point is outside the cuboid, the returned point will be on the surface of the cuboid.
    /// Otherwise, it will be inside the cuboid and returned as is.
    #[inline(always)]
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        // Clamp point coordinates to the cuboid
        point.clamp(-self.half_size, self.half_size)
    }
}

/// A cylinder primitive
#[derive(Clone, Copy, Debug)]
pub struct Cylinder {
    /// The radius of the cylinder
    pub radius: f32,
    /// The half height of the cylinder
    pub half_height: f32,
}
impl Primitive3d for Cylinder {}

impl Cylinder {
    /// Create a cylinder from a radius and full height
    pub fn new(radius: f32, height: f32) -> Self {
        Self {
            radius,
            half_height: height / 2.,
        }
    }
}

/// A capsule primitive.
/// A capsule is defined as a surface at a distance (radius) from a line
#[derive(Clone, Copy, Debug)]
pub struct Capsule {
    /// The radius of the capsule
    pub radius: f32,
    /// Half the height of the capsule, excluding the hemispheres
    pub half_length: f32,
}
impl super::Primitive2d for Capsule {}
impl Primitive3d for Capsule {}

impl Capsule {
    /// Create a new `Capsule` from a radius and length
    pub fn new(radius: f32, length: f32) -> Self {
        Self {
            radius,
            half_length: length / 2.0,
        }
    }
}

/// A cone primitive.
#[derive(Clone, Copy, Debug)]
pub struct Cone {
    /// The radius of the base
    pub radius: f32,
    /// The height of the cone
    pub height: f32,
}
impl Primitive3d for Cone {}

/// A conical frustum primitive.
/// A conical frustum can be created
/// by slicing off a section of a cone.
#[derive(Clone, Copy, Debug)]
pub struct ConicalFrustum {
    /// The radius of the top of the frustum
    pub radius_top: f32,
    /// The radius of the base of the frustum
    pub radius_bottom: f32,
    /// The height of the frustum
    pub height: f32,
}
impl Primitive3d for ConicalFrustum {}

/// The type of torus determined by the minor and major radii
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TorusKind {
    /// A torus that has a ring.
    /// The major radius is greater than the minor radius
    Ring,
    /// A torus that has no hole but also doesn't intersect itself.
    /// The major radius is equal to the minor radius
    Horn,
    /// A self-intersecting torus.
    /// The major radius is less than the minor radius
    Spindle,
    /// A torus with non-geometric properties like
    /// a minor or major radius that is non-positive,
    /// infinite, or `NaN`
    Invalid,
}

/// A torus primitive, often representing a ring or donut shape
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torus {
    /// The radius of the tube of the torus
    #[doc(
        alias = "ring_radius",
        alias = "tube_radius",
        alias = "cross_section_radius"
    )]
    pub minor_radius: f32,
    /// The distance from the center of the torus to the center of the tube
    #[doc(alias = "radius_of_revolution")]
    pub major_radius: f32,
}
impl Primitive3d for Torus {}

impl Torus {
    /// Create a new `Torus` from an inner and outer radius.
    ///
    /// The inner radius is the radius of the hole, and the outer radius
    /// is the radius of the entire object
    pub fn new(inner_radius: f32, outer_radius: f32) -> Self {
        let minor_radius = (outer_radius - inner_radius) / 2.0;
        let major_radius = outer_radius - minor_radius;

        Self {
            minor_radius,
            major_radius,
        }
    }

    /// Get the inner radius of the torus.
    /// For a ring torus, this corresponds to the radius of the hole,
    /// or `major_radius - minor_radius`
    #[inline]
    pub fn inner_radius(&self) -> f32 {
        self.major_radius - self.minor_radius
    }

    /// Get the outer radius of the torus.
    /// This corresponds to the overall radius of the entire object,
    /// or `major_radius + minor_radius`
    #[inline]
    pub fn outer_radius(&self) -> f32 {
        self.major_radius + self.minor_radius
    }

    /// Get the [`TorusKind`] determined by the minor and major radii.
    ///
    /// The torus can either be a *ring torus* that has a hole,
    /// a *horn torus* that doesn't have a hole but also isn't self-intersecting,
    /// or a *spindle torus* that is self-intersecting.
    ///
    /// If the minor or major radius is non-positive, infinite, or `NaN`,
    /// [`TorusKind::Invalid`] is returned
    #[inline]
    pub fn kind(&self) -> TorusKind {
        // Invalid if minor or major radius is non-positive, infinite, or NaN
        if self.minor_radius <= 0.0
            || !self.minor_radius.is_finite()
            || self.major_radius <= 0.0
            || !self.major_radius.is_finite()
        {
            return TorusKind::Invalid;
        }

        match self.major_radius.partial_cmp(&self.minor_radius).unwrap() {
            std::cmp::Ordering::Greater => TorusKind::Ring,
            std::cmp::Ordering::Equal => TorusKind::Horn,
            std::cmp::Ordering::Less => TorusKind::Spindle,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn direction_creation() {
        assert_eq!(Direction3d::new(Vec3::X * 12.5), Ok(Direction3d::X));
        assert_eq!(
            Direction3d::new(Vec3::new(0.0, 0.0, 0.0)),
            Err(InvalidDirectionError::Zero)
        );
        assert_eq!(
            Direction3d::new(Vec3::new(f32::INFINITY, 0.0, 0.0)),
            Err(InvalidDirectionError::Infinite)
        );
        assert_eq!(
            Direction3d::new(Vec3::new(f32::NEG_INFINITY, 0.0, 0.0)),
            Err(InvalidDirectionError::Infinite)
        );
        assert_eq!(
            Direction3d::new(Vec3::new(f32::NAN, 0.0, 0.0)),
            Err(InvalidDirectionError::NaN)
        );
        assert_eq!(
            Direction3d::new_and_length(Vec3::X * 6.5),
            Ok((Direction3d::X, 6.5))
        );
    }

    #[test]
    fn cuboid_closest_point() {
        let cuboid = Cuboid::new(2.0, 2.0, 2.0);
        assert_eq!(cuboid.closest_point(Vec3::X * 10.0), Vec3::X);
        assert_eq!(cuboid.closest_point(Vec3::NEG_ONE * 10.0), Vec3::NEG_ONE);
        assert_eq!(
            cuboid.closest_point(Vec3::new(0.25, 0.1, 0.3)),
            Vec3::new(0.25, 0.1, 0.3)
        );
    }

    #[test]
    fn sphere_closest_point() {
        let sphere = Sphere { radius: 1.0 };
        assert_eq!(sphere.closest_point(Vec3::X * 10.0), Vec3::X);
        assert_eq!(
            sphere.closest_point(Vec3::NEG_ONE * 10.0),
            Vec3::NEG_ONE.normalize()
        );
        assert_eq!(
            sphere.closest_point(Vec3::new(0.25, 0.1, 0.3)),
            Vec3::new(0.25, 0.1, 0.3)
        );
    }
}
