use chrono::{DateTime, Duration, Local, format::{DelayedFormat, StrftimeItems}};

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
                duration.num_minutes() % 60
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
