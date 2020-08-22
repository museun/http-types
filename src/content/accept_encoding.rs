//! Client header advertising available compression algorithms.

use crate::content::EncodingProposal;
use crate::headers::{HeaderName, HeaderValue, Headers, ToHeaderValues, ACCEPT_ENCODING};

use std::fmt::{self, Debug, Write};
use std::option;
use std::slice;

/// Client header advertising available compression algorithms.
pub struct AcceptEncoding {
    wildcard: bool,
    entries: Vec<EncodingProposal>,
}

impl AcceptEncoding {
    /// Create a new instance of `AcceptEncoding`.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            wildcard: false,
        }
    }

    /// Create an instance of `AcceptEncoding` from a `Headers` instance.
    pub fn from_headers(headers: impl AsRef<Headers>) -> crate::Result<Option<Self>> {
        let mut entries = vec![];
        let headers = match headers.as_ref().get(ACCEPT_ENCODING) {
            Some(headers) => headers,
            None => return Ok(None),
        };

        let mut wildcard = false;

        for value in headers {
            for part in value.as_str().trim().split(',') {
                let part = part.trim();

                // Handle empty strings, and wildcard directives.
                if part.is_empty() {
                    continue;
                } else if part == "*" {
                    wildcard = true;
                    continue;
                }

                // Try and parse a directive from a str. If the directive is
                // unkown we skip it.
                if let Some(entry) = EncodingProposal::from_str(part)? {
                    entries.push(entry);
                }
            }
        }

        Ok(Some(Self { entries, wildcard }))
    }

    /// Push a directive into the list of entries.
    pub fn push(&mut self, prop: impl Into<EncodingProposal>) {
        self.entries.push(prop.into());
    }

    /// Returns `true` if a wildcard directive was passed.
    pub fn wildcard(&self) -> bool {
        self.wildcard
    }

    /// Set the wildcard directive.
    pub fn set_wildcard(&mut self, wildcard: bool) {
        self.wildcard = wildcard
    }

    /// Insert a `HeaderName` + `HeaderValue` pair into a `Headers` instance.
    pub fn apply(&self, mut headers: impl AsMut<Headers>) {
        headers.as_mut().insert(ACCEPT_ENCODING, self.value());
    }

    /// Get the `HeaderName`.
    pub fn name(&self) -> HeaderName {
        ACCEPT_ENCODING
    }

    /// Get the `HeaderValue`.
    pub fn value(&self) -> HeaderValue {
        let mut output = String::new();
        for (n, directive) in self.entries.iter().enumerate() {
            let directive: HeaderValue = directive.clone().into();
            match n {
                0 => write!(output, "{}", directive).unwrap(),
                _ => write!(output, ", {}", directive).unwrap(),
            };
        }

        if self.wildcard {
            match output.len() {
                0 => write!(output, "*").unwrap(),
                _ => write!(output, ", *").unwrap(),
            }
        }

        // SAFETY: the internal string is validated to be ASCII.
        unsafe { HeaderValue::from_bytes_unchecked(output.into()) }
    }

    /// An iterator visiting all entries.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    /// An iterator visiting all entries.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.entries.iter_mut(),
        }
    }
}

impl IntoIterator for AcceptEncoding {
    type Item = EncodingProposal;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a AcceptEncoding {
    type Item = &'a EncodingProposal;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut AcceptEncoding {
    type Item = &'a mut EncodingProposal;
    type IntoIter = IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A borrowing iterator over entries in `AcceptEncoding`.
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<EncodingProposal>,
}

impl Iterator for IntoIter {
    type Item = EncodingProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A lending iterator over entries in `AcceptEncoding`.
#[derive(Debug)]
pub struct Iter<'a> {
    inner: slice::Iter<'a, EncodingProposal>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a EncodingProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over entries in `AcceptEncoding`.
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: slice::IterMut<'a, EncodingProposal>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut EncodingProposal;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ToHeaderValues for AcceptEncoding {
    type Iter = option::IntoIter<HeaderValue>;
    fn to_header_values(&self) -> crate::Result<Self::Iter> {
        // A HeaderValue will always convert into itself.
        Ok(self.value().to_header_values().unwrap())
    }
}

impl Debug for AcceptEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for directive in &self.entries {
            list.entry(directive);
        }
        list.finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::content::Encoding;
    use crate::Response;

    #[test]
    fn smoke() -> crate::Result<()> {
        let mut accept = AcceptEncoding::new();
        accept.push(Encoding::Gzip);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = AcceptEncoding::from_headers(headers)?.unwrap();
        assert_eq!(accept.iter().next().unwrap(), Encoding::Gzip);
        Ok(())
    }

    #[test]
    fn wildcard() -> crate::Result<()> {
        let mut accept = AcceptEncoding::new();
        accept.set_wildcard(true);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = AcceptEncoding::from_headers(headers)?.unwrap();
        assert!(accept.wildcard());
        Ok(())
    }

    #[test]
    fn wildcard_and_header() -> crate::Result<()> {
        let mut accept = AcceptEncoding::new();
        accept.push(Encoding::Gzip);
        accept.set_wildcard(true);

        let mut headers = Response::new(200);
        accept.apply(&mut headers);

        let accept = AcceptEncoding::from_headers(headers)?.unwrap();
        assert!(accept.wildcard());
        assert_eq!(accept.iter().next().unwrap(), Encoding::Gzip);
        Ok(())
    }
}
