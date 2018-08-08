use failure::Error;

pub struct SubcodeData {
    pub sectors: Vec<Sector>,
}

impl SubcodeData {
    pub fn parse(data: Vec<u8>) -> Result<SubcodeData, Error> {
        let mut sectors = vec![];
        for sector in data.as_slice().chunks(96) {
            sectors.push(Sector::parse(sector.to_vec())?);
        }

        Ok(SubcodeData {
            sectors: sectors,
        })
    }
}

#[derive(Debug, Fail)]
pub enum InvalidSectorError {
    #[fail(display = "invalid sector size; attempted to process subcode {}", index)]
    InvalidSubcodeIndex {
        index: usize,
    }
}

pub struct Sector {
    pub codes: Vec<Subcode>,
}

impl Sector {
    /// Parses a 96-byte `Vec` and returns a `Sector` whose data
    /// contains 8 12-byte `Subcode`s.
    pub fn parse(data: Vec<u8>) -> Result<Sector, InvalidSectorError> {
        let mut codes = vec![];

        for (i, data) in data.as_slice().chunks(12).enumerate() {
            let code;
            match SubcodeType::from_index(i) {
                Some(c) => code = c,
                None    => return Err(InvalidSectorError::InvalidSubcodeIndex { index: i }),
            }
            let mut data_vec = vec![];
            data_vec.extend_from_slice(data);
            codes.push(Subcode {
                channel: code,
                data: data_vec,
            });
        }

        Ok(Sector {
            codes: codes,
        })
    }

    /// Checks whether a subcode contains any non-basic subcodes -
    /// that is, any subcodes defined outside the CD-DA or CD-ROM
    /// specification. This method returns true if the R through V
    /// subcodes are empty, and false if they contain data.
    pub fn contains_basic_data_only(&self) -> bool {
        self.codes.iter().filter(|code| match code.channel {
                      SubcodeType::P | SubcodeType::Q => false,
                      _ => true,
                  })
                  .all(|code| code.is_empty())
    }
}

#[derive(Debug)]
pub enum SubcodeType {
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
}

impl SubcodeType {
    fn from_index(index: usize) -> Option<SubcodeType> {
        match index {
            0 => Some(SubcodeType::P),
            1 => Some(SubcodeType::Q),
            2 => Some(SubcodeType::R),
            3 => Some(SubcodeType::S),
            4 => Some(SubcodeType::T),
            5 => Some(SubcodeType::U),
            6 => Some(SubcodeType::V),
            7 => Some(SubcodeType::W),
            _ => None,
        }
    }
}

pub struct Subcode {
    pub channel: SubcodeType,
    pub data: Vec<u8>,
}

impl Subcode {
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|byte| byte == &0)
    }
}
