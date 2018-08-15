#[derive(Debug, Fail)]
pub enum InvalidDataError {
    #[fail(display = "invalid data size; must be a multiple of 96 bytes, was {}", length)]
    InvalidSubcodeDataLength {
        length: usize,
    },

    #[fail(display = "invalid sector size; attempted to process subcode {}", index)]
    InvalidSubcodeIndex {
        index: usize,
    },

    #[fail(display = "invalid sector size; must be exactly 96 bytes, was {}", length)]
    InvalidSectorLength {
        length: usize,
    },
}

pub struct SubcodeData {
    pub sectors: Vec<Sector>,
}

impl SubcodeData {
    pub fn parse(data: Vec<u8>) -> Result<SubcodeData, InvalidDataError> {
        if data.len() % 96 != 0 {
            return Err(InvalidDataError::InvalidSubcodeDataLength { length: data.len() });
        }

        let mut sectors = vec![];
        for sector in data.as_slice().chunks(96) {
            sectors.push(Sector::parse(sector.to_vec())?);
        }

        Ok(SubcodeData {
            sectors: sectors,
        })
    }

    pub fn contains_basic_data_only(&self) -> bool {
        self.sectors.iter().all(|sector| sector.contains_basic_data_only())
    }
}

pub struct Sector {
    pub codes: Vec<Subcode>,
}

impl Sector {
    /// Parses a 96-byte `Vec` and returns a `Sector` whose data
    /// contains 8 12-byte `Subcode`s.
    pub fn parse(data: Vec<u8>) -> Result<Sector, InvalidDataError> {
        let mut codes = vec![];

        // Each channel is 12 bytes, and there must be exactly 8 channels of data
        if data.len() != 96 {
            return Err(InvalidDataError::InvalidSectorLength { length: data.len() });
        }

        for (i, data) in data.as_slice().chunks(12).enumerate() {
            let code;
            match SubcodeType::from_index(i) {
                Some(c) => code = c,
                None    => return Err(InvalidDataError::InvalidSubcodeIndex { index: i }),
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

    /// Returns `Vec<SubcodeType>` indicating every channel for which
    /// this subcode contains data.
    /// Given data from a standard CD-ROM disc image, this will typically
    /// return a `Vec` with `P` and `Q`, but for discs containing extended
    /// data this will contain more.
    pub fn contains_data_in_channels(&self) -> Vec<SubcodeType> {
        let mut identities = vec![];

        for (index, code) in self.codes.iter().enumerate() {
            if code.is_empty() {
                continue
            }
            // We unwrap here because at the time this has been called,
            // we've validated that this data can only contain
            // precisely 8 channels. The error condition is unreachable.
            identities.push(SubcodeType::from_index(index).unwrap());
        }

        identities
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

    pub fn to_string(&self) -> String {
        match *self {
            SubcodeType::P => String::from("P"),
            SubcodeType::Q => String::from("Q"),
            SubcodeType::R => String::from("R"),
            SubcodeType::S => String::from("S"),
            SubcodeType::T => String::from("T"),
            SubcodeType::U => String::from("U"),
            SubcodeType::V => String::from("V"),
            SubcodeType::W => String::from("W"),
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

#[cfg(test)]
mod tests {
    use subcode;

    #[test]
    fn test_parsing_data_size() {
        let data1 = vec![0; 5];
        assert!(subcode::SubcodeData::parse(data1).is_err());

        let data2 = vec![0; 96];
        assert!(!subcode::SubcodeData::parse(data2).is_err());
    }

    #[test]
    fn test_invalid_sector_length() {
        let data = vec![];
        assert!(subcode::Sector::parse(data).is_err());
    }

    #[test]
    fn test_empty_subcode() {
        let subcode = subcode::Subcode {
            channel: subcode::SubcodeType::P,
            data: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        assert!(subcode.is_empty());
    }

    #[test]
    fn test_non_empty_subcode() {
        let subcode = subcode::Subcode {
            channel: subcode::SubcodeType::P,
            data: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        assert!(!subcode.is_empty());
    }

    #[test]
    fn test_basic_data_only() {
        let p = vec![1; 12];
        let q = vec![1; 12];
        let rest = vec![0; 72];
        let mut data = vec![];
        data.extend_from_slice(&p);
        data.extend_from_slice(&q);
        data.extend_from_slice(&rest);
        assert_eq!(96, data.len());

        let sector = subcode::Sector::parse(data).unwrap();
        assert!(sector.contains_basic_data_only());
    }

    #[test]
    fn test_contains_non_basic_data() {
        let data = vec![1; 96];
        assert_eq!(96, data.len());

        let sector = subcode::Sector::parse(data).unwrap();
        assert!(!sector.contains_basic_data_only());
    }

    #[test]
    fn test_basic_data_for_a_full_disc() {
        // Disc containing two sectors, both containing only basic data
        let sector1_p = vec![1; 12];
        let sector1_q = vec![1; 12];
        let sector1_rest = vec![0; 72];
        let sector2_p = vec![1; 12];
        let sector2_q = vec![1; 12];
        let sector2_rest = vec![0; 72];
        let mut data = vec![];
        data.extend_from_slice(&sector1_p);
        data.extend_from_slice(&sector1_q);
        data.extend_from_slice(&sector1_rest);
        data.extend_from_slice(&sector2_p);
        data.extend_from_slice(&sector2_q);
        data.extend_from_slice(&sector2_rest);
        assert_eq!(192, data.len());

        let subcode_data_result = subcode::SubcodeData::parse(data);
        assert!(subcode_data_result.is_ok());
        let subcode_data = subcode_data_result.unwrap();
        assert!(subcode_data.contains_basic_data_only());
    }

    #[test]
    fn test_non_basic_data_for_a_full_disc() {
        // Disc containing two sectors, both containing non-basic data
        let data = vec![1; 960];
        let subcode_data_result = subcode::SubcodeData::parse(data);
        assert!(subcode_data_result.is_ok());
        let subcode_data = subcode_data_result.unwrap();
        assert!(!subcode_data.contains_basic_data_only());
    }

    #[test]
    fn test_mixed_sector_modes_reports_correctly() {
        // Disc containing two sectors, one basic and one non-basic
        let sector1_p = vec![1; 12];
        let sector1_q = vec![1; 12];
        let sector1_rest = vec![0; 72];
        let sector2 = vec![1; 96];
        let mut data = vec![];
        data.extend_from_slice(&sector1_p);
        data.extend_from_slice(&sector1_q);
        data.extend_from_slice(&sector1_rest);
        data.extend_from_slice(&sector2);

        let subcode_data_result = subcode::SubcodeData::parse(data);
        assert!(subcode_data_result.is_ok());
        let subcode_data = subcode_data_result.unwrap();
        assert!(!subcode_data.contains_basic_data_only());
    }

    #[test]
    fn test_identifying_fields_from_a_basic_data_sector() {
        let sector1_p = vec![1; 12];
        let sector1_q = vec![1; 12];
        let sector1_rest = vec![0; 72];
        let mut data = vec![];
        data.extend_from_slice(&sector1_p);
        data.extend_from_slice(&sector1_q);
        data.extend_from_slice(&sector1_rest);

        let sector = subcode::Sector::parse(data).unwrap();
        assert!(sector.contains_basic_data_only());
        assert_eq!(2, sector.contains_data_in_channels().len());
    }

    #[test]
    fn test_identifying_fields_from_a_full_sector() {
        let data = vec![1; 96];

        let sector = subcode::Sector::parse(data).unwrap();
        assert!(!sector.contains_basic_data_only());
        assert_eq!(8, sector.contains_data_in_channels().len());
    }

    #[test]
    fn test_subcode_type_to_string() {
        assert_eq!("Q", subcode::SubcodeType::Q.to_string());
    }
}
