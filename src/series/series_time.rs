use crate::{
    chart::*,
    coord::{Axes, Stick},
    utils::cal_step::CalStep,
};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, ParseError};
use std::vec;

#[derive(Debug, Clone)]
/// A series of numbers represented on a chart
pub struct STime {
    series: Vec<NaiveDateTime>,
    format: String,
    dirty: bool,
    unit: String,
}

impl Default for STime {
    fn default() -> Self {
        Self {
            series: Default::default(),
            format: "%Y-%m-%d %H:%M:%S".to_string(),
            dirty: false,
            unit: "full".to_string(),
        }
    }
}

impl STime {
    pub fn new(series: Vec<NaiveDateTime>) -> Self {
        // let domain = min_max_vec(&series);
        STime {
            series,
            format: "%Y-%m-%d %H:%M:%S".to_string(),
            dirty: false,
            unit: "full".to_string(),
        }
    }

    pub fn set_format(&self, format: &str) -> Self {
        STime {
            series: self.series.clone(),
            format: format.to_string(),
            dirty: self.dirty,
            unit: "full".to_string(),
        }
    }

    pub fn set_data(&self, series: Vec<NaiveDateTime>) -> Self {
        STime {
            series: series,
            format: self.format.clone(),
            dirty: false,
            unit: "full".to_string(),
        }
    }

    pub fn get_unit(&self) -> String {
        self.unit.clone()
    }

    pub fn get_format(&self) -> &str {
        match self.unit.as_str() {
            "date" => "%Y-%m-%d",
            "month" => "%Y-%m",
            "year" => "%Y",
            "time" => "%H:%M:%S",
            "hour" => "%H:%M:%S",
            _ => "",
        }
    }

    pub fn series(&self) -> Vec<NaiveDateTime> {
        self.series.clone()
    }

    pub fn get_value(&self, index: usize) -> f64 {
        match self.unit.as_str() {
            // "date" => "%Y-%m-%d",
            // "month" => "%Y-%m",
            "year" => self.series[index].year() as f64,
            // "time" => "%H:%M:%S",
            // "hour" => "%H:%M:%S",
            _ => 1.,
        }
    }
}

impl From<(Vec<&str>, &str, &str)> for STime {
    fn from(value: (Vec<&str>, &str, &str)) -> Self {
        let vec = value.0;
        let format = value.1.to_string();
        let mut series: Vec<NaiveDateTime> = vec![];
        let mut dirty = false;
        let unit = value.2.to_string();
        for i in 0..vec.len() {
            let rndt = ndt_parse_from_str(vec[i], value.1, value.2);
            match rndt {
                Ok(ndt) => series.push(ndt),
                Err(_) => dirty = true,
            }
        }
        STime {
            series,
            format,
            dirty,
            unit,
        }
    }
}

fn ndt_parse_from_str(str: &str, format: &str, get: &str) -> Result<NaiveDateTime, ParseError> {
    match get {
        "full" => NaiveDateTime::parse_from_str(str, format),
        "date" => {
            let date = NaiveDate::parse_from_str(str, format);
            match date {
                Ok(d) => {
                    let time = NaiveTime::default();
                    Ok(NaiveDateTime::new(d, time))
                }
                Err(e) => Err(e),
            }
        }
        "year" => {
            let year = format!("{}-01-01", str);
            ndt_parse_from_str(year.as_str(), "%Y-%m-%d", "date")
        }

        _ => NaiveDateTime::parse_from_str(str, format),
    }
}

impl ScaleTime for STime {
    fn domain(&self) -> (NaiveDateTime, NaiveDateTime) {
        if self.series().len() > 0 {
            let binding = self.series();
            let min = binding.iter().min().unwrap();
            let max = binding.iter().max().unwrap();
            (*min, *max)
        } else {
            (NaiveDateTime::default(), NaiveDateTime::default())
        }
    }

    fn domain_unix(&self) -> (f64, f64) {
        let (min, max) = self.domain();
        match self.unit.as_str() {
            "year" => (min.year() as f64, max.year() as f64),
            _ => (0., 0.),
        }
    }

    fn count_distance_step(&self) -> (f64, f64) {
        match self.get_unit().as_str() {
            "year" => {
                let (min, max) = self.domain_unix();
                let dur = max - min;
                dbg!(&dur);
                let mut step = dur as f64 / 5.;
                step = CalStep::new(step.ceil()).cal_scale();
                dbg!(step);

                (dur as f64 / step, step)
            }
            // TODO: for month, day, time
            _ => (1., 0.),
        }
    }

    // fn get_intervale(&self, len: f64) -> f64 {
    //     let (distance, _step) = self.count_distance_step();
    //     len / distance
    // }

    fn scale_intervale(&self, value: NaiveDateTime) -> f64 {
        let (min, _max) = self.domain();
        let unit = self.get_unit();
        match unit.as_str() {
            "year" => (value.year() - min.year()) as f64,
            _ => (value.year() - min.year()) as f64,
        }
    }

    fn scale(&self, value: NaiveDateTime) -> f64 {
        let unit = self.get_unit();
        match unit.as_str() {
            "year" => {
                let (min, max) = self.domain_unix();
                let range = max - min;

                let diff = value.year() as f64 - min;
                diff / range
            }
            _ => 1.,
        }
    }

    fn gen_axes(&self) -> Axes {
        let series = self.series();
        let mut vec_stick: Vec<Stick> = vec![];
        // let interger_part = distance as u32;
        // dbg!(interger_part);
        let format = self.get_format();
        let unit = self.get_unit();
        // for index in 0..(series.len()) {
        match unit.as_str() {
            "year" => {
                for index in 0..(series.len()) {
                    let value = series[index];
                    let stick = Stick::new(value.format(format).to_string(), self.scale(value));
                    vec_stick.push(stick);
                }
            }
            _ => (),
        }

        let sticks = vec_stick
            .into_iter()
            .filter(|stick| stick.value >= -0.0000001 && stick.value <= 1.0000001)
            .collect::<Vec<_>>();

        Axes {
            sticks: sticks,
            step: 1.,
        }
    }

    fn to_stick(&self) -> Vec<Stick> {
        vec![]
    }
}
