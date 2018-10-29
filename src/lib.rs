#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]

use std::io;

/// A function for decoding a hex encoded string to Vec<u8>, leading zero agnostic
pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, io::Error> {
    let data = input.as_ref();
    if data.is_empty() {
        return Ok(vec![]);
    }
    strip_preamble(data)
        .into_iter()
        .map(|u| hex(*u))
        .collect::<Result<Vec<_>, io::Error>>()
        .map(|v| v.into_iter().skip_while(|u| *u == 0).collect::<Vec<_>>())
        .map(|mut v| {
            v.reverse(); // reverse for trailing zero handling
            v.chunks(2)
                .map(|c| if c.len() == 1 { c[0] } else { c[1] * 16 + c[0] })
                .rev() // rev back
                .collect::<Vec<_>>()
        })
}

// strip 0x
fn strip_preamble(data: &[u8]) -> &[u8] {
    if data.len() >= 2 && data[0] == 0x30 && data[1] == 0x78 {
        &data[2..]
    } else {
        data
    }
}

// convert ASCII
fn hex(byte: u8) -> Result<u8, io::Error> {
    if byte >= 0x30 && byte <= 0x39 {
        return Ok(byte - 0x30);
    }
    if byte >= 0x41 && byte <= 0x46 {
        return Ok(byte - 0x41 + 0xa);
    }
    if byte >= 0x61 && byte <= 0x66 {
        return Ok(byte - 0x61 + 0xa);
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Did not supply a properly encoded hex value",
    ))
}

#[cfg(test)]
mod tests {
    extern crate quickcheck;
    use super::decode;
    use quickcheck::TestResult;

    #[quickcheck]
    fn decoding_is_identity(input: Vec<u8>) -> TestResult {
        let mut encoded = String::with_capacity(input.len());
        let mut zero_check = true;

        for byte in &input {
            encoded.push_str(&format!("{:0>2x}", byte));
            if *byte != 0 {
                zero_check = false;
            }
        }
        if input.is_empty() {
            let empty: Vec<u8> = vec![];
            return TestResult::from_bool(empty == decode(encoded.as_bytes()).unwrap());
        }
        if zero_check {
            return TestResult::from_bool(vec![0] == decode(encoded.as_bytes()).unwrap());
        }
        TestResult::from_bool(
            input
                .into_iter()
                .skip_while(|u| *u == 0)
                .collect::<Vec<_>>()
                == decode(encoded.as_bytes()).unwrap(),
        )
    }

    #[quickcheck]
    fn decoding_is_identity_with_prefix(input: Vec<u8>) -> TestResult {
        let mut encoded = String::with_capacity(input.len() + 2);
        encoded.push_str("0x");
        let mut zero_check = true;

        for byte in &input {
            encoded.push_str(&format!("{:0>2x}", byte));
            if *byte != 0 {
                zero_check = false;
            }
        }
        if input.is_empty() {
            let empty: Vec<u8> = vec![];
            return TestResult::from_bool(empty == decode(encoded.as_bytes()).unwrap());
        }
        if zero_check {
            return TestResult::from_bool(vec![0] == decode(encoded.as_bytes()).unwrap());
        }
        TestResult::from_bool(
            input
                .into_iter()
                .skip_while(|u| *u == 0)
                .collect::<Vec<_>>()
                == decode(encoded.as_bytes()).unwrap(),
        )
    }
}
