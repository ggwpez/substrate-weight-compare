use serde::{Deserialize, Serialize};

pub trait One {
	fn one() -> Self;
}

pub trait Zero {
	fn zero() -> Self;
}

impl One for u128 {
	fn one() -> Self {
		1
	}
}

impl Zero for u128 {
	fn zero() -> Self {
		0
	}
}

impl ValueFormatter for u128 {
	fn format_scalar(&self) -> String {
		crate::Dimension::fmt_scalar(*self)
	}
}

pub trait ValueFormatter {
	fn format_scalar(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct Weight {
	pub time: u128,
	pub proof: u128,
}

impl Zero for Weight {
	fn zero() -> Self {
		Self { time: 0, proof: 0 }
	}
}

impl One for Weight {
	fn one() -> Self {
		Self { time: 1, proof: 1 }
	}
}

impl From<(u128, u128)> for Weight {
	fn from((time, proof): (u128, u128)) -> Self {
		Self { time, proof }
	}
}

impl Into<(u128, u128)> for Weight {
	fn into(self) -> (u128, u128) {
		(self.time, self.proof)
	}
}

impl From<u128> for Weight {
	fn from(time: u128) -> Self {
		Self { time, proof: 0 }
	}
}

impl ValueFormatter for Weight {
	fn format_scalar(&self) -> String {
		format!("({}, {})", self.time, self.proof)
	}
}

impl core::fmt::Display for Weight {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "({}, {})", self.time, self.proof)
	}
}

impl Weight {
	pub fn mul_scalar(&self, other: u128) -> Self {
		Self {
			time: self.time * other,
			proof: self.proof * other,
		}
	}
}
