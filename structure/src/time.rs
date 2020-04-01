use std::cmp::Ordering;
use std::fmt::{Display, Formatter,};
use std::ops::{Add, Div, Mul, Rem, Sub};

use num_traits::{AsPrimitive, PrimInt, Unsigned};

use crate::time::fmt::TimeFormat;
use crate::time::TimeUnit::{Days, Hours, Minutes, Months, Weeks, Years};

pub type YearsType = u16;
pub type FineGrainTimeType = usize;

pub mod fmt {
    use std::fmt::{Display, Formatter, Result};
    use std::str::FromStr;

    use regex::{Captures, Regex};

    use crate::time::{Time, TimeUnit};
    use crate::time::TimeUnit::{Days, Hours, Minutes, Months, Weeks, Years};

    pub struct TimeFormat<'a, 'b> {
        reference: &'a TimeUnit,
        format_string: &'b str,
    }

    pub trait TimeFormatArgs<'a> {
        fn get_format_string(&self) -> &'a str;
    }

    impl<'a> TimeFormatArgs<'a> for &'a str {
        fn get_format_string(&self) -> &'a str {
            self
        }
    }

    impl<'a, 'b> TimeFormat<'a, 'b> {
        pub fn new(reference: &'a TimeUnit, format_string: &'b str) -> Self {
            TimeFormat {
                reference,
                format_string,
            }
        }

        fn formatted_time_string(captures: &Captures, numerator: TimeUnit) -> String {
            if let Some(c) = captures.get(2) {
                let unit = captures.get(3).unwrap().as_str();
                if let Ok(quantity) = usize::from_str(c.as_str()) {
                    let denominator = match unit {
                        "m" => Minutes(quantity),
                        "h" => Hours(quantity),
                        "d" => Days(quantity),
                        "w" => Weeks(quantity),
                        "M" => Months(quantity),
                        "y" => Years(quantity as u16),
                        _ => {
                            panic!("Divisor type must be [mhdwMy], found {}", unit);
                        }
                    };
                    let fixed = numerator % denominator;
                    format!("{}", fixed)
                } else {
                    panic!("Must be an integer")
                }
            } else {
                format!("{}", numerator)
            }
        }
    }

    impl<'a, 'b> Display for TimeFormat<'a, 'b> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            let output = self.format_string;

            let output = &*Regex::new("\\{:m(\\((\\d+)([mhdwMy])\\))?}")
                .expect("Regular expression forming failed")
                .replace_all(&output, |captures: &Captures| -> String {
                    let numerator = self.reference.as_minutes();
                    Self::formatted_time_string(captures, numerator)
                });

            let output = &*Regex::new("\\{:h(\\((\\d+)([mhdwMy])\\))?}")
                .expect("Regular expression forming failed")
                .replace_all(&output, |captures: &Captures| -> String {
                    let numerator = self.reference.as_hours();
                    Self::formatted_time_string(captures, numerator)
                });

            let output = &*Regex::new("\\{:d(\\((\\d+)([mhdwMy])\\))?}")
                .expect("Regular expression forming failed")
                .replace_all(&output, |captures: &Captures| -> String {
                    let numerator = self.reference.as_days();
                    Self::formatted_time_string(captures, numerator)
                });

            let output = &*Regex::new("\\{:w(\\((\\d+)([mhdwMy])\\))?}")
                .expect("Regular expression forming failed")
                .replace_all(&output, |captures: &Captures| -> String {
                    let numerator = self.reference.as_weeks();
                    Self::formatted_time_string(captures, numerator)
                });

            let output = &*Regex::new("\\{:M(\\((\\d+)([mhdwMy])\\))?}")
                .expect("Regular expression forming failed")
                .replace_all(&output, |captures: &Captures| -> String {
                    let numerator = self.reference.as_months();
                    Self::formatted_time_string(captures, numerator)
                });

            let output = &*Regex::new("\\{:y(\\((\\d+)([mhdwMy])\\))?}")
                .expect("Regular expression forming failed")
                .replace_all(&output, |captures: &Captures| -> String {
                    let numerator = self.reference.as_years();
                    Self::formatted_time_string(captures, numerator)
                });

            write!(f, "{}", output)
        }
    }

    pub struct DefaultAge;
    pub struct DefaultTime;
}

#[derive(Clone, Debug)]
pub enum TimeUnit {
    Minutes(FineGrainTimeType),
    Hours(FineGrainTimeType),
    Days(FineGrainTimeType),
    Weeks(FineGrainTimeType),
    Months(FineGrainTimeType),
    Years(YearsType),
}

impl TimeUnit {
    fn as_minutes(&self) -> TimeUnit {
        Minutes(match self {
            Minutes(min) => *min,
            Hours(hrs) => *hrs * 60,
            Days(days) => *days * 24 * 60,
            Months(months) => ((*months as f64) * 30.42) as FineGrainTimeType * 24 * 60,
            Years(yrs) => (*yrs as usize * 365) as FineGrainTimeType * 24 * 60,
            Weeks(w) => w * 7 * 24 * 60,
        })
    }

    fn resolution_val(&self) -> u8 {
        match self {
            Minutes(_) => 6,
            Hours(_) => 5,
            Days(_) => 4,
            Weeks(_) => 3,
            Months(_) => 2,
            Years(_) => 1,
        }
    }

    fn cmp_resolution(&self, other: &Self) -> Ordering {
        self.resolution_val().cmp(&other.resolution_val())
    }

    pub fn format(&self, format_string: &str) -> String {
        let form = TimeFormat::new(self, format_string);
        format!("{}", form)
    }
}

pub trait Time: Into<usize> + PartialOrd<usize> + Clone {
    fn into_minutes(self) -> TimeUnit;
    fn into_hours(self) -> TimeUnit;
    fn into_days(self) -> TimeUnit;
    fn into_weeks(self) -> TimeUnit;
    fn into_months(self) -> TimeUnit;
    fn into_years(self) -> TimeUnit;
    fn as_minutes(&self) -> TimeUnit {
        let next = self.clone();
        next.into_minutes()
    }
    fn as_hours(&self) -> TimeUnit {
        let next = self.clone();
        next.into_hours()
    }
    fn as_days(&self) -> TimeUnit {
        let next = self.clone();
        next.into_days()
    }
    fn as_weeks(&self) -> TimeUnit {
        let next = self.clone();
        next.into_weeks()
    }
    fn as_months(&self) -> TimeUnit {
        let next = self.clone();
        next.into_months()
    }
    fn as_years(&self) -> TimeUnit {
        let next = self.clone();
        next.into_years()
    }
}

impl From<TimeUnit> for usize {
    /// Returns the backing value of the TimeUnit
    fn from(unit: TimeUnit) -> Self {
        match unit {
            Minutes(t) | Hours(t) | Days(t) | Weeks(t) | Months(t) => t,
            Years(t) => t as usize,
        }
    }
}

impl From<&TimeUnit> for usize {
    /// Returns the backing value of the TimeUnit
    fn from(unit: &TimeUnit) -> Self {
        match unit {
            Minutes(t) | Hours(t) | Days(t) | Weeks(t) | Months(t) => *t,
            Years(t) => *t as usize,
        }
    }
}

impl Time for TimeUnit {
    fn into_minutes(self) -> TimeUnit {
        TimeUnit::as_minutes(&self)
    }

    fn into_hours(self) -> TimeUnit {
        Hours(usize::from(self.into_minutes()) / 60)
    }

    fn into_days(self) -> TimeUnit {
        Days(usize::from(self.into_minutes()) / 60 / 24)
    }

    fn into_weeks(self) -> TimeUnit {
        Weeks(usize::from(self.into_minutes()) / 60 / 24 / 7)
    }

    fn into_months(self) -> TimeUnit {
        Months(usize::from((self.into_minutes()) / 60 / 24 / 30.42))
    }

    fn into_years(self) -> TimeUnit {
        Years(usize::from((self.into_minutes() / 60 / 24 / 365)) as YearsType)
    }
}

impl Rem for TimeUnit {
    type Output = TimeUnit;

    fn rem(self, rhs: Self) -> Self::Output {
        match rhs {
            Minutes(m) => Minutes(usize::from(self.into_minutes()) % m),
            Hours(h) => Hours(usize::from(self.into_hours()) % h),
            Days(d) => Days(usize::from(self.into_days()) % d),
            Weeks(w) => Weeks(usize::from(self.into_weeks()) % w),
            Months(m) => Months(usize::from(self.into_months()) % m),
            Years(y) => Years(usize::from(self.into_years()) as u16 % y),
        }
    }
}

impl Mul<usize> for TimeUnit {
    type Output = TimeUnit;

    fn mul(self, rhs: usize) -> Self::Output {
        match self {
            Minutes(min) => Minutes(min * rhs),
            Hours(hrs) => Hours(hrs * rhs),
            Days(days) => Days(days * rhs),
            Weeks(wks) => Weeks(wks * rhs),
            Months(months) => Months(months * rhs),
            Years(years) => Years(years * rhs as YearsType),
        }
    }
}

impl Div<usize> for TimeUnit {
    type Output = TimeUnit;

    fn div(self, rhs: usize) -> Self::Output {
        match self {
            Minutes(min) => Minutes(min / rhs),
            Hours(hrs) => Hours(hrs / rhs),
            Days(days) => Days(days / rhs),
            Weeks(wks) => Weeks(wks / rhs),
            Months(months) => Months(months / rhs),
            Years(years) => Years(years / rhs as YearsType),
        }
    }
}

impl Mul<f64> for TimeUnit {
    type Output = TimeUnit;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Minutes(min) => Minutes((min as f64 * rhs) as FineGrainTimeType),
            Hours(hrs) => Hours((hrs as f64 * rhs) as FineGrainTimeType),
            Days(days) => Days((days as f64 * rhs) as FineGrainTimeType),
            Weeks(wks) => Weeks((wks as f64 * rhs) as FineGrainTimeType),
            Months(months) => Months((months as f64 * rhs) as FineGrainTimeType),
            Years(years) => Years((years as f64 * rhs) as YearsType),
        }
    }
}

impl Div<f64> for TimeUnit {
    type Output = TimeUnit;

    fn div(self, rhs: f64) -> Self::Output {
        match self {
            Minutes(min) => Minutes((min as f64 / rhs).round() as FineGrainTimeType),
            Hours(hrs) => Hours((hrs as f64 / rhs).round() as FineGrainTimeType),
            Days(days) => Days((days as f64 / rhs).round() as FineGrainTimeType),
            Weeks(wks) => Weeks((wks as f64 / rhs).round() as FineGrainTimeType),
            Months(months) => Months((months as f64 / rhs).round() as FineGrainTimeType),
            Years(years) => Years((years as f64 / rhs).round() as YearsType),
        }
    }
}

impl Add<TimeUnit> for FineGrainTimeType {
    type Output = FineGrainTimeType;

    fn add(self, rhs: TimeUnit) -> Self::Output {
        self + (match rhs {
            Minutes(t) | Hours(t) | Days(t) | Weeks(t) | Months(t) => t,
            Years(t) => t as FineGrainTimeType,
        })
    }
}

impl Add<TimeUnit> for YearsType {
    type Output = YearsType;

    fn add(self, rhs: TimeUnit) -> Self::Output {
        if let Years(yrs) = rhs {
            self + yrs
        } else {
            self + rhs.into_years()
        }
    }
}

impl Sub<TimeUnit> for FineGrainTimeType {
    type Output = FineGrainTimeType;

    fn sub(self, rhs: TimeUnit) -> Self::Output {
        self - (match rhs {
            Minutes(t) | Hours(t) | Days(t) | Weeks(t) | Months(t) => t,
            Years(t) => t as FineGrainTimeType,
        })
    }
}

impl Sub<TimeUnit> for YearsType {
    type Output = YearsType;

    fn sub(self, rhs: TimeUnit) -> Self::Output {
        if let Years(yrs) = rhs {
            self - yrs
        } else {
            self - rhs.into_years()
        }
    }
}

impl Add<TimeUnit> for TimeUnit {
    type Output = Self;

    ///
    /// Adds two TimeUnits together, results in a TimeUnit with the greatest Resolution
    fn add(self, rhs: TimeUnit) -> Self::Output {
        match self.cmp_resolution(&rhs) {
            Ordering::Less => {
                // Communitive if using resolution fixing
                rhs + self
            }
            Ordering::Greater | Ordering::Equal => match self {
                Minutes(min) => Minutes(min + rhs.into_minutes()),
                Hours(hrs) => Hours(hrs + rhs.into_hours()),
                Days(days) => Days(days + rhs.into_days()),
                Weeks(wks) => Weeks(wks + rhs.into_weeks()),
                Months(months) => Months(months + rhs.into_months()),
                Years(years) => Years(years + rhs),
            },
        }
    }
}

impl Sub<TimeUnit> for TimeUnit {
    type Output = Self;

    ///
    /// Adds two TimeUnits together, results in a TimeUnit with the greatest Resolution
    fn sub(self, rhs: TimeUnit) -> Self::Output {
        match self.cmp_resolution(&rhs) {
            Ordering::Less => {
                // Communitive if using resolution fixing
                rhs - self
            }
            Ordering::Greater | Ordering::Equal => match self {
                Minutes(min) => Minutes(min - rhs.into_minutes()),
                Hours(hrs) => Hours(hrs - rhs.into_hours()),
                Days(days) => Days(days - rhs.into_days()),
                Weeks(wks) => Weeks(wks - rhs.into_weeks()),
                Months(months) => Months(months - rhs.into_months()),
                Years(years) => Years(years - rhs),
            },
        }
    }
}

impl Sub<&TimeUnit> for TimeUnit {
    type Output = Self;

    fn sub(self, rhs: &TimeUnit) -> Self::Output {
        self - rhs.clone()
    }
}

impl Add<&TimeUnit> for TimeUnit {
    type Output = Self;

    fn add(self, rhs: &TimeUnit) -> Self::Output {
        self + rhs.clone()
    }
}

impl<T> Add<T> for TimeUnit
where
    T: PrimInt + Unsigned + AsPrimitive<FineGrainTimeType>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        match self {
            Minutes(min) => Minutes(min + rhs.as_()),
            Hours(hrs) => Hours(hrs + rhs.as_()),
            Days(days) => Days(days + rhs.as_()),
            Weeks(wks) => Weeks(wks + rhs.as_()),
            Months(months) => Months(months + rhs.as_()),
            Years(years) => Years(years + rhs.as_() as YearsType),
        }
    }
}

impl Add<TimeUnit> for &TimeUnit {
    type Output = TimeUnit;

    fn add(self, rhs: TimeUnit) -> Self::Output {
        self.clone() + rhs
    }
}

impl Add<&TimeUnit> for &TimeUnit {
    type Output = TimeUnit;

    fn add(self, rhs: &TimeUnit) -> Self::Output {
        self + rhs.clone()
    }
}

impl<T> Add<T> for &TimeUnit
where
    T: PrimInt + Unsigned + AsPrimitive<FineGrainTimeType>,
{
    type Output = TimeUnit;

    fn add(self, rhs: T) -> Self::Output {
        match self.clone() {
            Minutes(min) => Minutes(min + rhs.as_()),
            Hours(hrs) => Hours(hrs + rhs.as_()),
            Days(days) => Days(days + rhs.as_()),
            Weeks(wks) => Weeks(wks + rhs.as_()),
            Months(months) => Months(months + rhs.as_()),
            Years(years) => Years(years + rhs.as_() as YearsType),
        }
    }
}

impl PartialEq<usize> for TimeUnit {
    fn eq(&self, other: &usize) -> bool {
        usize::from(self).eq(other)
    }
}

impl PartialOrd<usize> for TimeUnit {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        usize::from(self).partial_cmp(other)
    }
}

impl PartialEq<TimeUnit> for TimeUnit {
    fn eq(&self, other: &TimeUnit) -> bool {
        self.as_minutes().eq(&usize::from(other.as_minutes()))
    }
}

impl PartialOrd<TimeUnit> for TimeUnit {
    fn partial_cmp(&self, other: &TimeUnit) -> Option<Ordering> {
        self.as_minutes()
            .partial_cmp(&usize::from(other.as_minutes()))
    }
}

impl PartialEq<TimeUnit> for &TimeUnit {
    fn eq(&self, other: &TimeUnit) -> bool {
        self.as_minutes().eq(&usize::from(other.as_minutes()))
    }
}

impl PartialOrd<TimeUnit> for &TimeUnit {
    fn partial_cmp(&self, other: &TimeUnit) -> Option<Ordering> {
        self.as_minutes()
            .partial_cmp(&usize::from(other.as_minutes()))
    }
}

impl PartialEq<&TimeUnit> for TimeUnit {
    fn eq(&self, other: &&TimeUnit) -> bool {
        self.as_minutes().eq(&usize::from(other.as_minutes()))
    }
}

impl PartialOrd<&TimeUnit> for TimeUnit {
    fn partial_cmp(&self, other: &&TimeUnit) -> Option<Ordering> {
        self.as_minutes()
            .partial_cmp(&usize::from(other.as_minutes()))
    }
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", usize::from(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_conversion() {
        let base = Days(32);
        assert_eq!(base.into_hours(), 32 * 24);
    }

    #[test]
    fn into_consistency() {
        let minutes = Minutes(755);
        assert_eq!(minutes.as_minutes(), minutes);
        let hours = Hours(255);
        assert_eq!(hours.as_hours(), hours);
        let days = Days(755);
        assert_eq!(days.as_days(), days);
        let weeks = Weeks(25);
        assert_eq!(weeks.as_weeks(), weeks);
        let months = Months(14);
        assert_eq!(months.as_months(), months);
        let years = Years(255);
        assert_eq!(years.as_years(), years);
    }

    #[test]
    fn add_time_unit() {
        let base = Days(32) + Months(1);
        assert_eq!(base, Days(62));
        let base = Hours(15) + Minutes(120);
        assert_eq!(base, Hours(17));
    }

    #[test]
    fn resolution_scope() {
        let a = Minutes(15) + Hours(1);
        assert_eq!(a, 75);
        let b = Hours(1) + Minutes(15);
        if let Minutes(_) = b {
        } else {
            panic!("Resolution should scope to Minutes, scoped to {:?}", b)
        }
        assert_eq!(a, b);
        let c = Years(5) + Hours(1);
        if let Hours(_) = c {
        } else {
            panic!("Resolution should scope to Hours, scoped to {:?}", c)
        }
    }

    #[test]
    fn compare() {
        let lhs = Days(5);
        let rhs = Days(7);
        assert_ne!(lhs, rhs);
        assert!(lhs < rhs);
        assert!(rhs >= lhs);

        let rhs = Days(5);
        assert_eq!(lhs, rhs);
        assert!(lhs >= rhs);
        assert!(lhs <= rhs);
        assert!(!(lhs < rhs));
        assert!(!(lhs > rhs));

        let lhs = Minutes(15);
        assert_ne!(lhs, rhs);
        assert!(lhs < rhs);
        assert!(rhs >= lhs);

        let lhs = Minutes(5);
        // testing to see that comparing two times with same usize::from value aren't equal if their
        // Grain is different
        assert_ne!(lhs, rhs);
        assert!(lhs < rhs);
    }

    #[test]
    fn time_remain() {
        let a = Months(12);
        let b = Months(12);
        assert_eq!(a % b.clone(), Months(0));
        let a = Months(15);
        assert_eq!(a % b.clone(), Months(3));
    }

    #[test]
    fn time_format() {
        let age = Years(21) + Days(150) + Hours(25) + Minutes(45);
        let age_string = age.format("{:y} years {:d(365d)} days");
        assert_eq!(age_string, "21 years 151 days");

        let time = Hours(41) + Minutes(23);
        let time_string = time.format("{:h}:{:m(60m)}");
        assert_eq!(time_string, "41:23");
    }
}
