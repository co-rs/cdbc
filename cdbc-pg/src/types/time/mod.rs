use ::time::Month;
use mco::std::lazy::sync::Lazy;

mod date;
mod datetime;
mod time;

#[rustfmt::skip]
const  PG_EPOCH: Lazy<::time::Date> = Lazy::new(|| ::time::Date::from_calendar_date(2000, Month::January, 1).unwrap());
