use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Duration, Local,
};

pub enum Timer {
    Measuring(DateTime<Local>),
    Ended(DateTime<Local>, DateTime<Local>),
}
impl Timer {
    const TIME_FORMAT: &'static str = "%m/%d %H:%M:%S";

    pub fn start() -> Self {
        Self::Measuring(Local::now())
    }

    pub fn finish(&mut self) {
        match self {
            Self::Measuring(start_time) => *self = Self::Ended(*start_time, Local::now()),
            Self::Ended(_, end_time) => *end_time = Local::now(),
        }
    }

    pub fn formatted_start(&self) -> DelayedFormat<StrftimeItems<'_>> {
        match self {
            Self::Measuring(start_time) => start_time.format(Self::TIME_FORMAT),
            Self::Ended(start_time, _) => start_time.format(Self::TIME_FORMAT),
        }
    }

    pub fn formatted_end(&self) -> DelayedFormat<StrftimeItems<'_>> {
        match self {
            Self::Measuring(_) => Local::now().format(Self::TIME_FORMAT),
            Self::Ended(_, end_time) => end_time.format(Self::TIME_FORMAT),
        }
    }

    fn duration(&self) -> Duration {
        match self {
            Self::Measuring(start_time) => Local::now() - *start_time,
            Self::Ended(start_time, end_time) => *end_time - *start_time,
        }
    }

    pub fn formatted_duration(&self) -> String {
        let duration = self.duration();
        if duration.num_hours() > 0 {
            format!("{}h{}m", duration.num_hours(), duration.num_minutes() % 60)
        } else if duration.num_minutes() > 0 {
            format!(
                "{}m{}s",
                duration.num_minutes(),
                duration.num_seconds() % 60
            )
        } else {
            format!(
                "{}.{:>02}s",
                duration.num_seconds(),
                duration.num_milliseconds() % 1000 / 10
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::*;

    #[test]
    fn test_measuring_start_format() {
        let timer = Timer::Measuring(Local.ymd(2019, 5, 3).and_hms(13, 59, 20));
        assert_eq!(
            "05/03 13:59:20".to_string(),
            timer.formatted_start().to_string(),
        );
    }

    #[test]
    fn test_ended_start_format() {
        let timer = Timer::Ended(
            Local.ymd(2019, 5, 3).and_hms(13, 59, 20),
            Local.ymd(2021, 9, 4).and_hms(18, 43, 10),
        );
        assert_eq!(
            "05/03 13:59:20".to_string(),
            timer.formatted_start().to_string(),
        );
    }

    #[test]
    fn test_ended_end_format() {
        let timer = Timer::Ended(
            Local.ymd(2019, 5, 3).and_hms(13, 59, 20),
            Local.ymd(2021, 9, 4).and_hms(18, 43, 10),
        );
        assert_eq!(
            "09/04 18:43:10".to_string(),
            timer.formatted_end().to_string(),
        );
    }

    #[test]
    fn test_ended_duration_format_over_1_hour() {
        let timer = Timer::Ended(
            Local.ymd(2019, 5, 3).and_hms(13, 50, 20),
            Local.ymd(2019, 5, 3).and_hms(18, 53, 30),
        );
        assert_eq!("5h3m".to_string(), timer.formatted_duration(),);
    }

    #[test]
    fn test_ended_duration_format_between_1_min_with_1_hour() {
        let timer = Timer::Ended(
            Local.ymd(2019, 5, 3).and_hms(13, 50, 20),
            Local.ymd(2019, 5, 3).and_hms(13, 53, 30),
        );
        assert_eq!("3m10s".to_string(), timer.formatted_duration(),);
    }

    #[test]
    fn test_ended_duration_format_under_1_min() {
        let timer = Timer::Ended(
            Local.ymd(2019, 5, 3).and_hms_milli(13, 50, 20, 490),
            Local.ymd(2019, 5, 3).and_hms_milli(13, 51, 9, 601),
        );
        assert_eq!("49.11s".to_string(), timer.formatted_duration(),);
    }
}
