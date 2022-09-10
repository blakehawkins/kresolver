use std::ops::{AddAssign, Neg, SubAssign};
use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Serialize, Serializer};

/// A currency amount as used in ledgers. Reads and writes with precision to
/// four places past the decimal. Internally uses a bigint representation with
/// fixed size.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Amount(Decimal);

impl Amount {
    fn new(amount: Decimal) -> Self {
        Amount(amount)
    }
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let res = format!("{}", self.0.round_dp(4));
        serializer.serialize_str(&res)
    }
}

impl FromStr for Amount {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s).map(Amount::new)
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Neg for Amount {
    type Output = Amount;

    fn neg(self) -> Self::Output {
        Amount(-self.0)
    }
}
