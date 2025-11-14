use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

#[derive(Eq, PartialEq)]
pub struct ImageCounter(pub usize);

impl Display for ImageCounter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.0 {
            0 => f.write_str("no images"),
            1 => f.write_str("1 image"),
            _ => write!(f, "{} images", self.0),
        }
    }
}
