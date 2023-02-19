// type specifiers
pub const SIMPLE_STRING: u8 = b'+';
pub const ERROR: u8 = b'-';
pub const INTEGER: u8 = b':';
pub const BULK_STRING: u8 = b'$';
pub const ARRAY: u8 = b'*';
pub const NULL: u8 = b'_';
pub const DOUBLE: u8 = b',';
pub const BOOLEAN: u8 = b'#';
pub const BULK_ERROR: u8 = b'!';
pub const VERBATIM_STRING: u8 = b'=';
pub const BIG_INTEGER: u8 = b'(';
pub const MAP: u8 = b'%';
pub const SET: u8 = b'~';
pub const ATTRIBUTE: u8 = b'|';
pub const PUSH: u8 = b'>';
pub const STREAM: u8 = b';';
pub const END: u8 = b'.';

// byte used in types
pub const UNSPECIFIED_SIZE: u8 = b'?';
pub const VERBATIM_STRING_SEPARATOR: u8 = b':';

// const bytes
pub const HELLO: [u8; 5] = [b'H', b'E', b'L', b'L', b'O'];
pub const AUTH: [u8; 4] = [b'A', b'U', b'T', b'H'];
pub const TRUE: [u8; 2] = [BOOLEAN, b't'];
pub const FALSE: [u8; 2] = [BOOLEAN, b'f'];
pub const INFINITE: [u8; 4] = [DOUBLE, b'i', b'n', b'f'];
pub const NEG_INFINITE: [u8; 5] = [DOUBLE, b'-', b'i', b'n', b'f'];
pub const NAN: [u8; 4] = [DOUBLE, b'n', b'a', b'n'];
pub const STREAM_END: [u8; 2] = [STREAM, b'0'];
pub const NEWLINE: [u8; 2] = [b'\r', b'\n'];
