use crate::simple_fs::SimpleString;

pub trait StringExt {
    fn repeat(&self, count: usize) -> SimpleString;
    fn to_uppercase(&self) -> SimpleString;
    fn to_lowercase(&self) -> SimpleString;
}

pub trait StringSliceExt {
    fn join(&self, separator: &str) -> SimpleString;
}

impl<T: AsRef<str>> StringSliceExt for [T] {
    fn join(&self, separator: &str) -> SimpleString {
        let mut result = SimpleString::new();
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                result.push_str(separator);
            }
            result.push_str(item.as_ref());
        }
        result
    }
}

impl StringExt for str {
    fn repeat(&self, count: usize) -> SimpleString {
        let mut result = SimpleString::new();
        for _ in 0..count {
            result.push_str(self);
        }
        result
    }

    fn to_uppercase(&self) -> SimpleString {
        let mut result = SimpleString::new();
        for c in self.chars() {
            if c >= 'a' && c <= 'z' {
                result.push((c as u8 - 32) as char);
            } else {
                result.push(c);
            }
        }
        result
    }

    fn to_lowercase(&self) -> SimpleString {
        let mut result = SimpleString::new();
        for c in self.chars() {
            if c >= 'A' && c <= 'Z' {
                result.push((c as u8 + 32) as char);
            } else {
                result.push(c);
            }
        }
        result
    }
}

impl StringExt for SimpleString {
    fn repeat(&self, count: usize) -> SimpleString {
        self.as_str().repeat(count)
    }

    fn to_uppercase(&self) -> SimpleString {
        self.as_str().to_uppercase()
    }

    fn to_lowercase(&self) -> SimpleString {
        self.as_str().to_lowercase()
    }
} 