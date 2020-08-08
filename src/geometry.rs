use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
};

pub type Scalar = f32;
pub const PI: f32 = std::f32::consts::PI;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vector3 {
	pub x: Scalar,
	pub y: Scalar,
	pub z: Scalar,
}

impl Vector3 {
	pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
		Self { x, y, z }
	}

	pub fn zero() -> Self {
		Self::new(0.0, 0.0, 0.0)
	}

	pub fn len(&self) -> Scalar {
		(self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
	}

	pub fn normalized(&self) -> Self {
		if self.len() == 0.0 {
			return *self;
		}

		*self / self.len()
	}

	pub fn as_slice(&self) -> [Scalar; 3] {
		[self.x, self.y, self.z]
	}

	pub fn as_matrix4x4(&self) -> Matrix4x4 {
		#[rustfmt::skip]
		return Matrix4x4::from_values([
			self.x, 0.0, 0.0, 0.0,
			self.y, 0.0, 0.0, 0.0,
			self.z, 0.0, 0.0, 0.0,
			1.0, 0.0, 0.0, 0.0,
		]);
	}

	pub fn mix(&self, other: Vector3, amount: Scalar) -> Self {
		*self * (1.0 - amount) + other * amount
	}

	pub fn rotated(&self, x: Scalar, y: Scalar, z: Scalar) -> Self {
		let rotation_matrix = Matrix4x4::identity().rotated(x, y, z);
		*self * rotation_matrix
	}

	pub fn dot(&self, other: Vector3) -> Scalar {
		self.x * other.x + self.y * other.y + self.z * other.z
	}
}

pub fn vec3(x: Scalar, y: Scalar, z: Scalar) -> Vector3 {
	Vector3::new(x, y, z)
}

impl Add for Vector3 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl Sub for Vector3 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl AddAssign for Vector3 {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl Mul<Scalar> for Vector3 {
	type Output = Self;

	fn mul(self, rhs: Scalar) -> Self {
		Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
	}
}

impl Mul<Matrix4x4> for Vector3 {
	type Output = Self;

	fn mul(self, rhs: Matrix4x4) -> Self {
		let result = rhs * self.as_matrix4x4();
		let result_vec =
			Vector3::new(*result.at(0, 0), *result.at(1, 0), *result.at(2, 0));
		result_vec / *result.at(3, 0)
	}
}

impl Div<Scalar> for Vector3 {
	type Output = Self;

	fn div(self, rhs: Scalar) -> Self {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}

impl DivAssign<Scalar> for Vector3 {
	fn div_assign(&mut self, rhs: Scalar) {
		*self = *self / rhs;
	}
}

impl Neg for Vector3 {
	type Output = Self;

	fn neg(self) -> Self {
		self * -1.0
	}
}

impl std::fmt::Display for Vector3 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "vec3 {{ {:8.5}, {:8.5}, {:8.5} }}", self.x, self.y, self.z)?;
		Ok(())
	}
}


#[derive(Copy, Clone)]
pub struct Matrix4x4 {
	values: [Scalar; 16],
}

impl Matrix4x4 {
	pub fn zero() -> Self {
		Self::from_values([0.0; 16])
	}

	pub fn identity() -> Self {
		#[rustfmt::skip]
		return Self::from_values([
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]);
	}

	pub fn from_values(values: [Scalar; 16]) -> Self {
		Self { values }
	}

	pub fn at(&self, i: usize, j: usize) -> &Scalar {
		&self.values[i * 4 + j]
	}

	pub fn at_mut(&mut self, i: usize, j: usize) -> &mut Scalar {
		&mut self.values[i * 4 + j]
	}

	pub fn rotated(&self, x: Scalar, y: Scalar, z: Scalar) -> Self {
		#[rustfmt::skip]
		let x_rot_matrix = Matrix4x4::from_values([
			1.0, 0.0, 0.0, 0.0,
			0.0, x.cos(), -x.sin(), 0.0,
			0.0, x.sin(), x.cos(), 0.0,
			0.0, 0.0, 0.0, 1.0
		]);
		#[rustfmt::skip]
		let y_rot_matrix = Matrix4x4::from_values([
			y.cos(), 0.0, y.sin(), 0.0,
			0.0, 1.0, 0.0, 0.0,
			-y.sin(), 0.0, y.cos(), 0.0,
			0.0, 0.0, 0.0, 1.0
		]);
		#[rustfmt::skip]
		let z_rot_matrix = Matrix4x4::from_values([
			z.cos(), -z.sin(), 0.0, 0.0,
			z.sin(), z.cos(), 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]);

		*self * x_rot_matrix * y_rot_matrix * z_rot_matrix
	}

	pub fn transposed(&self) -> Self {
		let mut output = Matrix4x4::zero();
		for i in 0..4 {
			for j in 0..4 {
				*output.at_mut(i, j) = *self.at(j, i);
			}
		}
		output
	}

	pub fn translated_by_vec3(&self, vector: Vector3) -> Self {
		#[rustfmt::skip]
		let translation_matrix = Matrix4x4::from_values([
			1.0, 0.0, 0.0, vector.x,
			0.0, 1.0, 0.0, vector.y,
			0.0, 0.0, 1.0, vector.z,
			0.0, 0.0, 0.0, 1.0,
		]);
		// let translation_matrix = Matrix4x4::from_values([
		// 	1.0, 0.0, 0.0, 0.0,
		// 	0.0, 1.0, 0.0, 0.0,
		// 	0.0, 0.0, 1.0, 0.0,
		// 	vector.x, vector.y, vector.z, 1.0,
		// ]);

		*self * translation_matrix
	}
}

impl Mul for Matrix4x4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		let mut output = Matrix4x4::zero();

		for i in 0..4 {
			for j in 0..4 {
				let cell_value = (0..4)
					.map(|k| self.at(i, k) * rhs.at(k, j))
					.sum::<Scalar>();
				*output.at_mut(i, j) = cell_value;
			}
		}

		output
	}
}

impl MulAssign for Matrix4x4 {
	fn mul_assign(&mut self, rhs: Self) {
		*self = *self * rhs;
	}
}

impl std::fmt::Debug for Matrix4x4 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// TODO: Make this more idiomatic in how it uses `Formatter`.
		write!(f, "Matrix4x4 {{\n")?;
		for i in 0..4 {
			write!(f, "   {:6.2}, ", *self.at(i, 0))?;
			write!(f, "{:6.2}, ", *self.at(i, 1))?;
			write!(f, "{:6.2}, ", *self.at(i, 2))?;
			write!(f, "{:6.2}\n", *self.at(i, 3))?;
		}
		write!(f, "}}")?;
		Ok(())
	}
}

pub fn normal(a: Vector3, b: Vector3, c: Vector3) -> Vector3 {
	let v = b - a;
	let w = c - a;

	let normal_x = (v.y * w.z) - (v.z * w.y);
	let normal_y = (v.z * w.x) - (v.x * w.z);
	let normal_z = (v.x * w.y) - (v.y * w.x);

	Vector3::new(normal_x, normal_y, normal_z).normalized()
}

// pub fn clockwise(a: Vector3, b: Vector3, c: Vector3) -> bool {
	
// }
