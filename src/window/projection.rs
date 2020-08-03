use crate::geometry;

use geometry::{Matrix4x4, Scalar, Vector3};

pub struct Camera {
	pub position: Vector3,
	pub rotation: Vector3,
	pub fov_degrees: Scalar,
}

// cameraPos: vec(0, -5, 0),
// 			cameraRot: vec(0, 0, 0),
// 			fovDegrees: 40,

impl Camera {
	pub fn default() -> Self {
		Self {
			position: Vector3::new(0.0, 0.0, 2000.0),
			rotation: Vector3::new(0.0, 0.0, 0.0),
			fov_degrees: 40.0,
		}
	}

	pub fn new(
		position: Vector3,
		rotation: Vector3,
		fov_degrees: Scalar,
	) -> Self {
		Self {
			position,
			rotation,
			fov_degrees,
		}
	}

	pub fn projector(
		&self,
		viewport_width: Scalar,
		viewport_height: Scalar,
	) -> CameraProjector {
		let pmv_matrix = create_pmv_matrix(
			self.fov_degrees,
			self.position,
			self.rotation,
			viewport_width,
			viewport_height,
		);
		CameraProjector::new(pmv_matrix, viewport_width, viewport_height)
	}
}

pub struct CameraProjector {
	pmv_matrix: Matrix4x4,
	viewport_width: Scalar,
	viewport_height: Scalar,
}

impl CameraProjector {
	pub fn new(
		pmv_matrix: Matrix4x4,
		viewport_width: Scalar,
		viewport_height: Scalar,
	) -> Self {
		Self {
			pmv_matrix,
			viewport_width,
			viewport_height,
		}
	}

	#[inline(always)]
	pub fn project_point(&self, point: Vector3) -> (Scalar, Scalar, Scalar) {
		//println!("{:?}", pmv_matrix);

		// println!("Using: {:?}", pmv_matrix);
		// println!("Point: {:?}", point);
		let projected_position = point * self.pmv_matrix;
		// println!("->     {:?}", projected_position);

		let (px, py) = (projected_position.x, projected_position.y);
		let hw = self.viewport_width / 2.0;
		let hh = self.viewport_height / 2.0;

		(px * hw + hw, py * hh + hh, projected_position.z)
	}
}

fn create_pmv_matrix(
	fov_degrees: Scalar,
	position: Vector3,
	rotation: Vector3,
	viewport_width: Scalar,
	viewport_height: Scalar,
) -> Matrix4x4 {
	let aspect_ratio = viewport_width / viewport_height;

	let near = 0.1;
	let far = 50_000.0;

	let height = 2.0 * near * fov_degrees.to_radians().tan();
	let width = aspect_ratio * height;

	#[rustfmt::skip]
	let projection_matrix = Matrix4x4::from_values([
		2.0 * near / width, 0.0, 0.0, 0.0,
		0.0, 2.0 * near / height, 0.0, 0.0,
		0.0, 0.0, (far + near) / (near - far), 2.0 * far * near / (near - far),
		0.0, 0.0, -1.0, 0.0,
	]);

	let r = rotation;
	let model_view_matrix = Matrix4x4::identity().rotated(r.x, r.y, r.z);
	let model_view_matrix = model_view_matrix.translated_by_vec3(-position);

	let pmv_matrix = projection_matrix * model_view_matrix;
	//let pmv_matrix = pmv_matrix.transposed();

	// let mat = Matrix4x4::from_values([
	// 	1.0, 2.0, 3.0, 4.0,
	// 	5.0, 6.0, 7.0, 8.0,
	// 	9.0, 1.0, 2.0, 3.0,
	// 	4.0, 5.0, 6.0, 7.0,
	// ]);
	// let vec = Vector3::new(2.5, 3.5, 4.5);
	// println!("{:?}", vec);
	// std::process::exit(0);

	// println!("{} {}", viewport_width, viewport_height);
	// println!("{} {:?} {:?}", fov_degrees.to_radians(), position, rotation);
	// println!("{:?}", pmv_matrix);

	// std::process::exit(0);

	pmv_matrix
}
