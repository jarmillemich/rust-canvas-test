use std::ops;

pub struct FixedPoint(u64);

const FRACTION_WIDTH: u32 = 16;

impl From<u8> for FixedPoint {
    fn from(value: u8) -> Self {
        Self::from(value as u64)
    }
}

impl From<u16> for FixedPoint {
    fn from(value: u16) -> Self {
        Self::from(value as u64)
    }
}

impl From<u32> for FixedPoint {
    fn from(value: u32) -> Self {
        Self::from(value as u64)
    }
}

impl From<u64> for FixedPoint {
    fn from(value: u64) -> Self {
        let (res, overflow) = value.overflowing_shl(FRACTION_WIDTH);
        assert!(!overflow, "Fixed point overflow");
        Self(res)
    }
}

impl ops::Add<FixedPoint> for FixedPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign<FixedPoint> for FixedPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::Sub<FixedPoint> for FixedPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl ops::SubAssign<FixedPoint> for FixedPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl ops::Mul<FixedPoint> for FixedPoint {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // Expand out to a u64, and then bit-shift right again to align the point
        let mid = self.0 as u128 * rhs.0 as u128;
        Self((mid >> 32) as u32)
    }
}