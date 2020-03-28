use std::ops::Add;

pub const START_YEAR: u16 = 1900;

pub type YearsType = u16;
pub type TimeFineGrain = usize;


pub struct Minutes(pub TimeFineGrain);
pub struct Hours(pub TimeFineGrain);
pub struct Days(pub TimeFineGrain);
pub struct Months(pub TimeFineGrain);
pub struct Years(pub YearsType);

pub trait Time {
    fn into_minutes(self) -> Minutes;
}



impl From<Hours> for Minutes
{
    fn from(other: Hours) -> Self {
        Self(other.0 * 60)
    }
}

impl From<Days> for Minutes
{
    fn from(other: Days) -> Self {
        Self::from(Hours::from(other))
    }
}

impl From<Months> for Minutes
{
    fn from(other: Months) -> Self {
        Self::from(Hours::from(other))
    }
}

impl From<Years> for Minutes
{
    fn from(other: Years) -> Self {
        Self::from(Hours::from(other))
    }
}

impl Time for Minutes {
    fn into_minutes(self) -> Minutes {
        self
    }
}

impl From<Minutes> for Hours
{
    fn from(other: Minutes) -> Self {
        Self(other.0 / 60)
    }
}

impl From<Days> for Hours
{
    fn from(other: Days) -> Self {
        Self::from(Minutes::from(other))
    }
}

impl From<Months> for Hours
{
    fn from(other: Months) -> Self {
        Self::from(Minutes::from(other))
    }
}

impl From<Years> for  Hours
{
    fn from(other: Years) -> Self {
        Self::from(Minutes::from(other))
    }
}

impl Time for Hours {
    fn into_minutes(self) -> Minutes {
        Minutes::from(self)
    }
}

impl From<Minutes> for Days
{
    fn from(other: Minutes) -> Self {
        Self::from(Hours::from(other))
    }
}

impl From<Hours> for Days
{
    fn from(other: Hours) -> Self {
        Self(other.0 / 24)
    }
}

impl From<Months> for Days
{
    fn from(other: Months) -> Self {
        Self::from(Hours::from(other))
    }
}

impl From<Years> for Days
{
    fn from(other: Years) -> Self {
        Self::from(Hours::from(other))
    }
}

impl Time for Days {
    fn into_minutes(self) -> Minutes {
        Minutes::from(self)
    }
}

impl From<Minutes> for Months
{
    fn from(other: Minutes) -> Self {
        Self::from(Days::from(other))
    }
}

impl From<Hours> for Months
{
    fn from(other: Hours) -> Self {
        Self::from(Days::from(other))
    }
}

impl From<Days> for Months
{
    fn from(other: Days) -> Self {
        Self((other.0 as f64 / 30.42).round() as TimeFineGrain)
    }
}

impl From<Years> for Months
{
    fn from(other: Years) -> Self {
        Self::from(Days::from(other))
    }
}

impl Time for Months {
    fn into_minutes(self) -> Minutes {
        Minutes::from(self)
    }
}


impl From<Minutes> for Years
{
    fn from(other: Minutes) -> Self {
        Self::from(Days::from(other))
    }
}

impl From<Hours> for Years
{
    fn from(other: Hours) -> Self {
        Self::from(Days::from(other))
    }
}

impl From<Days> for Years
{
    fn from(other: Days) -> Self {
        Self((other.0 as f64 / 365.25).round() as YearsType)
    }
}

impl From<Months> for Years
{
    fn from(other: Months) -> Self {
        Self::from(Days::from(other))
    }
}



impl Time for Years {
    fn into_minutes(self) -> Minutes {
        Minutes::from(self)
    }
}


impl<R> Add<R> for Minutes
    where
          R : Time
{
    type Output = Self;

    fn add(self, rhs: R) -> Self::Output {
        let minutes_left: Minutes = self.into_minutes();
        let minutes_right: Minutes = rhs.into_minutes();
        let sum = minutes_left.0 + minutes_right.0;
        Self::Output::from(Minutes(sum))
    }
}



pub struct Age(Minutes);

impl Age {

    pub fn new(years: YearsType, months: TimeFineGrain, days: TimeFineGrain) -> Age {
        let years = Years(years).into();
        let months = Months(months);
        let days = Days(days);

        Age(years)
    }
}