use nom::{
    IResult, Parser,
    bytes::complete::{tag, take, take_until1},
    multi::many0,
};

#[derive(Debug)]
pub struct RedisProtocol {
    pub params_n: usize,
    pub params_list: Vec<RedisParam>,
}

#[derive(Debug)]
pub struct RedisParam {
    pub param_size: usize,
    pub param_value: String,
}

impl RedisProtocol {
    pub fn new(params_n: usize, params_list: Vec<RedisParam>) -> RedisProtocol {
        RedisProtocol {
            params_n,
            params_list,
        }
    }

    pub fn from_str(input: &str) -> IResult<&str, RedisProtocol> {
        let (input, params_n) = get_params_n(input)?;
        let (input, _) = take(2u8)(input)?;
        let (input, params_list) = get_params_list(input)?;
        let redis_protocol = RedisProtocol::new(params_n, params_list);
        Ok((input, redis_protocol))
    }

    pub fn valid(&self) -> bool {
        if self.params_n != self.params_list.len() {
            return false;
        }
        for param in &self.params_list {
            if param.param_size != param.param_value.len() {
                return false;
            }
        }
        true
    }
}

impl RedisParam {
    pub fn new(param_size: usize, param_value: String) -> RedisParam {
        RedisParam {
            param_size,
            param_value,
        }
    }
}

fn get_params_n(input: &str) -> IResult<&str, usize> {
    // Check for *
    let (input, _) = tag("*")(input)?;
    let (input, params_n_str) = take_until1("\r\n")(input)?;
    let params_n = params_n_str.parse::<usize>().unwrap();
    Ok((input, params_n))
}

fn get_param(input: &str) -> IResult<&str, RedisParam> {
    // Check for $
    let (input, _) = tag("$")(input)?;
    // Grab length
    let (input, param_size_str) = take_until1("\r\n")(input)?;
    // parse length
    let param_size = param_size_str.parse::<usize>().unwrap();
    let (input, _) = take(2u8)(input)?;

    // Move to next line
    let (input, param_value) = take_until1("\r\n")(input)?;
    let (input, _) = take(2u8)(input)?;
    let redisparam = RedisParam::new(param_size, param_value.to_string());
    Ok((input, redisparam))
}

fn get_params_list(input: &str) -> IResult<&str, Vec<RedisParam>> {
    many0(get_param).parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_bytes_to_redisprotocol_struct() {
        let str = "*1\r\n$4\r\nPING\r\n";
        let (_, protocol) = RedisProtocol::from_str(str).unwrap();
        assert_eq!(protocol.params_n, 1);
        assert_eq!(protocol.params_list[0].param_size, 4);
        assert_eq!(protocol.params_list[0].param_value, "PING".to_string());
        assert!(protocol.valid());
    }

    #[test]
    fn parse_bytes_to_fail_params_length() {
        let str = "*1\r\n$4\r\nPING\r\n$2\r\nHI\r\n";
        let (_, protocol) = RedisProtocol::from_str(str).unwrap();
        assert!(!protocol.valid());
    }

    #[test]
    fn parse_bytes_to_fail_param_size_length() {
        let str = "*1\r\n$5\r\nPING\r\n";
        let (_, protocol) = RedisProtocol::from_str(str).unwrap();
        assert!(!protocol.valid());
    }

    #[test]
    fn parse_echo() {
        let str = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
        let (_, protocol) = RedisProtocol::from_str(str).unwrap();
        assert_eq!(protocol.params_n, 2);
        assert_eq!(protocol.params_list[0].param_size, 4);
        assert_eq!(protocol.params_list[0].param_value, "ECHO".to_string());
        assert_eq!(protocol.params_list[1].param_size, 3);
        assert_eq!(protocol.params_list[1].param_value, "hey".to_string());
        assert!(protocol.valid());
    }
}
