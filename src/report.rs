#[path = "./traits.rs"]
mod traits;

use chrono::{DateTime, FixedOffset, Local, NaiveTime, Offset, TimeZone};
use std::fmt;
use traits::DateTimeExt;

#[derive(Debug)]
pub struct SolarReport {
    solar_noon: NaiveTime,
    sunrise: NaiveTime,
    sunset: NaiveTime,

    date: DateTime<FixedOffset>,
    latitude: f64,
    longitude: f64,
}

impl Default for SolarReport {
    fn default() -> SolarReport {
        let local_time = Local::now();
        SolarReport {
            solar_noon: NaiveTime::from_hms(0, 0, 0),
            sunrise: NaiveTime::from_hms(0, 0, 0),
            sunset: NaiveTime::from_hms(0, 0, 0),
            date: local_time.with_timezone(&FixedOffset::from_offset(local_time.offset())),
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

impl fmt::Display for SolarReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_str = format!(
            "LOCATION\n\
        --------\n\
        Latitude:  {}\n\
        Longitude: {}\n\n\
        DATE\n\
        ----\n\
        {}\n\n\
        Sunrise on this day is at {}\n\
        Sunset on this day is at  {}\n\
        Solar noon is at          {}",
            self.latitude, self.longitude, self.date, self.sunrise, self.sunset, self.solar_noon
        );

        write!(f, "{}", fmt_str)
    }
}

impl SolarReport {
    pub fn new(date: DateTime<FixedOffset>, latitude: f64, longitude: f64) -> SolarReport {
        let mut report = SolarReport {
            date,
            latitude,
            longitude,
            ..Default::default()
        };

        report.run();

        // make it immutable again by default
        let report = report;
        report
    }

    fn day_fraction_to_time(day_fraction: f64) -> NaiveTime {
        // day_fraction should be 0 <= day_fraction < 1
        // let day_fraction = day.fract();
        let hour_fraction = day_fraction * 24.0;
        let minute_fraction = hour_fraction.fract() * 60.0;
        let second_fraction = minute_fraction.fract() * 60.0;

        NaiveTime::from_hms(
            hour_fraction.trunc() as u32,
            minute_fraction.trunc() as u32,
            second_fraction.trunc() as u32,
        )
    }

    fn run(&mut self) {
        let time_zone = self.date.offset().fix().local_minus_utc() as f64 / 3600.0;

        let julian_date: f64 = self.date.to_julian_date();

        let julian_century = (julian_date - 2451545.0) / 36525.0;

        let geometric_solar_mean_longitude =
            (280.46646 + julian_century * (36000.76983 + julian_century * 0.0003032)) % 360.0;

        let solar_mean_anomaly =
            357.52911 + julian_century * (35999.05029 - 0.0001537 * julian_century);

        let eccent_earth_orbit =
            0.016708634 - julian_century * (0.000042037 + 0.0000001267 * julian_century);

        let equation_of_the_center = solar_mean_anomaly.to_radians().sin()
            * (1.914602 - julian_century * (0.004817 + 0.000014 * julian_century))
            + (2.0 * solar_mean_anomaly).to_radians().sin()
                * (0.019993 - 0.000101 * julian_century)
            + (3.0 * solar_mean_anomaly).to_radians().sin() * 0.000289;

        let solar_true_longitude = geometric_solar_mean_longitude + equation_of_the_center;

        let solar_apparent_longitude = solar_true_longitude
            - 0.00569
            - 0.00478 * (125.04 - 1934.136 * julian_century).to_radians().sin();

        let mean_oblique_ecliptic = 23.0
            + (26.0
                + (21.448
                    - julian_century
                        * (46.815 + julian_century * (0.00059 - julian_century * 0.001813)))
                    / 60.0)
                / 60.0;

        let oblique_corrected = mean_oblique_ecliptic
            + 0.00256 * (125.04 - 1934.136 * julian_century).to_radians().cos();

        let solar_declination = ((oblique_corrected.to_radians().sin()
            * solar_apparent_longitude.to_radians().sin())
        .asin())
        .to_degrees();

        let var_y = (oblique_corrected / 2.0).to_radians().tan().powi(2);

        let equation_of_time = 4.0
            * (var_y * (geometric_solar_mean_longitude.to_radians() * 2.0).sin()
                - 2.0 * eccent_earth_orbit * solar_mean_anomaly.to_radians().sin()
                + 4.0
                    * eccent_earth_orbit
                    * var_y
                    * solar_mean_anomaly.to_radians().sin()
                    * (geometric_solar_mean_longitude.to_radians() * 2.0).cos()
                - 0.5 * var_y * var_y * (geometric_solar_mean_longitude.to_radians() * 4.0).sin()
                - 1.25
                    * eccent_earth_orbit
                    * eccent_earth_orbit
                    * (solar_mean_anomaly.to_radians() * 2.0).sin())
            .to_degrees();

        let hour_angle = (((90.833f64.to_radians().cos()
            / (self.latitude.to_radians().cos() * solar_declination.to_radians().cos()))
            - self.latitude.to_radians().tan() * solar_declination.to_radians().tan())
        .acos())
        .to_degrees();

        let solar_noon =
            (720.0 - 4.0 * self.longitude - equation_of_time + time_zone * 60.0) / 1440.0;

        self.sunrise = SolarReport::day_fraction_to_time(
            ((solar_noon - hour_angle * 4.0) / 1440.0) + solar_noon,
        );
        self.sunset = SolarReport::day_fraction_to_time(
            ((solar_noon + hour_angle * 4.0) / 1440.0) + solar_noon,
        );
        self.solar_noon = SolarReport::day_fraction_to_time(solar_noon)
    }
}
