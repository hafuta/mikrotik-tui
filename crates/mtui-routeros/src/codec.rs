//! Length-prefixed `RouterOS` API words and sentences.

use crate::error::{Error, Result};

/// Upper bound on a single API word (command, attribute, or reply).
pub const MAX_WORD_BYTES: usize = 1 << 20;

/// Encode `words` as one API sentence, terminated by an empty word.
#[must_use]
pub fn encode_sentence(words: &[impl AsRef<str>]) -> Vec<u8> {
    let mut out = Vec::new();
    for word in words {
        encode_word(word.as_ref().as_bytes(), &mut out);
    }
    encode_word(&[], &mut out);
    out
}

fn encode_word(bytes: &[u8], out: &mut Vec<u8>) {
    let len = bytes.len();
    if len < 0x80 {
        out.push(u8::try_from(len).unwrap_or(0));
    } else if len < 0x4000 {
        out.push(0x80 | u8::try_from(len >> 8).unwrap_or(0));
        out.push(u8::try_from(len & 0xff).unwrap_or(0));
    } else if len < 0x20_0000 {
        out.push(0xc0 | u8::try_from(len >> 16).unwrap_or(0));
        out.push(u8::try_from((len >> 8) & 0xff).unwrap_or(0));
        out.push(u8::try_from(len & 0xff).unwrap_or(0));
    } else if len < 0x1000_0000 {
        out.push(0xe0 | u8::try_from(len >> 24).unwrap_or(0));
        out.push(u8::try_from((len >> 16) & 0xff).unwrap_or(0));
        out.push(u8::try_from((len >> 8) & 0xff).unwrap_or(0));
        out.push(u8::try_from(len & 0xff).unwrap_or(0));
    } else {
        out.push(0xf0);
        out.extend_from_slice(&u32::try_from(len).unwrap_or(u32::MAX).to_be_bytes());
    }
    out.extend_from_slice(bytes);
}

/// Decode one length-prefixed word from `buf`, returning `(word, bytes_consumed)`.
pub fn decode_word(buf: &[u8]) -> Result<Option<(Vec<u8>, usize)>> {
    if buf.is_empty() {
        return Ok(None);
    }
    let first = buf[0];
    let (len, header) = if first < 0x80 {
        (usize::from(first), 1usize)
    } else if first < 0xc0 {
        if buf.len() < 2 {
            return Ok(None);
        }
        let len = (usize::from(first & 0x3f) << 8) | usize::from(buf[1]);
        (len, 2)
    } else if first < 0xe0 {
        if buf.len() < 3 {
            return Ok(None);
        }
        let len =
            (usize::from(first & 0x1f) << 16) | (usize::from(buf[1]) << 8) | usize::from(buf[2]);
        (len, 3)
    } else if first < 0xf0 {
        if buf.len() < 4 {
            return Ok(None);
        }
        let len = (usize::from(first & 0x0f) << 24)
            | (usize::from(buf[1]) << 16)
            | (usize::from(buf[2]) << 8)
            | usize::from(buf[3]);
        (len, 4)
    } else if first == 0xf0 {
        if buf.len() < 5 {
            return Ok(None);
        }
        let len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
        (len, 5)
    } else {
        return Err(Error::decode(
            "decode_word",
            format!("unsupported length prefix {first:#04x}"),
        ));
    };
    if len > MAX_WORD_BYTES {
        return Err(Error::decode("decode_word", "API word too large"));
    }
    let total = header.saturating_add(len);
    if buf.len() < total {
        return Ok(None);
    }
    Ok(Some((buf[header..total].to_vec(), total)))
}

/// Incremental sentence decoder. Bytes are pushed with [`push`]; complete
/// sentences are taken with [`take_sentence`].
#[derive(Debug, Default)]
pub struct SentenceDecoder {
    buf: Vec<u8>,
    words: Vec<String>,
}

impl SentenceDecoder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    #[must_use]
    pub fn buffered_bytes(&self) -> usize {
        self.buf.len()
    }

    pub fn take_sentence(&mut self) -> Result<Option<Vec<String>>> {
        loop {
            let Some((word, consumed)) = decode_word(&self.buf)? else {
                return Ok(None);
            };
            self.buf.drain(..consumed);
            if word.is_empty() {
                let sentence = std::mem::take(&mut self.words);
                return Ok(Some(sentence));
            }
            // Lossy: a binary `/file` word must not abort the read loop, or the
            // tagged request waits until timeout with no `!done`.
            self.words.push(String::from_utf8_lossy(&word).into_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_short_words_and_empty_terminator() {
        let encoded = encode_sentence(&["/login", "=name=admin", "=password=secret"]);
        let mut decoder = SentenceDecoder::new();
        decoder.push(&encoded);
        let sentence = decoder.take_sentence().unwrap().unwrap();
        assert_eq!(sentence, vec!["/login", "=name=admin", "=password=secret"]);
        assert!(decoder.take_sentence().unwrap().is_none());
    }

    #[test]
    fn encodes_one_byte_length_prefix() {
        let encoded = encode_sentence(&["!re"]);
        assert_eq!(encoded[0], 3);
        assert_eq!(&encoded[1..4], b"!re");
        assert_eq!(encoded[4], 0);
    }

    #[test]
    fn roundtrips_two_byte_length_prefix() {
        let word = "x".repeat(200);
        let encoded = encode_sentence(&[word.as_str()]);
        assert_eq!(encoded[0], 0x80);
        assert_eq!(encoded[1], 200);
        let mut decoder = SentenceDecoder::new();
        decoder.push(&encoded);
        assert_eq!(decoder.take_sentence().unwrap().unwrap(), vec![word]);
    }

    #[test]
    fn captured_done_and_re_transcript() {
        let encoded =
            encode_sentence(&["!re", "=.id=*1", "=name=ether1", "=running=true", ".tag=7"]);
        let mut more = encode_sentence(&["!done", ".tag=7"]);
        let mut decoder = SentenceDecoder::new();
        decoder.push(&encoded);
        decoder.push(&more.split_off(0));
        assert_eq!(
            decoder.take_sentence().unwrap().unwrap(),
            vec!["!re", "=.id=*1", "=name=ether1", "=running=true", ".tag=7"]
        );
        assert_eq!(
            decoder.take_sentence().unwrap().unwrap(),
            vec!["!done", ".tag=7"]
        );
    }

    #[test]
    fn feeds_partial_bytes_until_sentence_completes() {
        let encoded = encode_sentence(&["!trap", "=message=failure: bad"]);
        let mut decoder = SentenceDecoder::new();
        decoder.push(&encoded[..3]);
        assert!(decoder.take_sentence().unwrap().is_none());
        decoder.push(&encoded[3..]);
        assert_eq!(
            decoder.take_sentence().unwrap().unwrap(),
            vec!["!trap", "=message=failure: bad"]
        );
    }

    #[test]
    fn keeps_non_utf8_words_so_the_sentence_can_finish() {
        let payload = b"=data=\xff";
        let mut encoded = vec![u8::try_from(payload.len()).expect("fits")];
        encoded.extend_from_slice(payload);
        encoded.push(0);
        let mut decoder = SentenceDecoder::new();
        decoder.push(&encoded);
        let sentence = decoder.take_sentence().unwrap().unwrap();
        assert_eq!(sentence.len(), 1);
        assert!(sentence[0].starts_with("=data="));
        assert!(decoder.take_sentence().unwrap().is_none());
    }

    #[test]
    fn rejects_oversized_word() {
        let mut buf = vec![0xf0, 0x00, 0x20, 0x00, 0x01];
        buf.extend(std::iter::repeat_n(b'a', 16));
        assert!(decode_word(&buf).is_err());
    }
}
