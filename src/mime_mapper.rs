use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

const CT_APPLICATION_OCTET_STREAM: &str = "Content-Type: application/octet-stream";

static MIME_MAPPINGS: LazyLock<MimeMapper> = LazyLock::new(MimeMapper::default);

type MimeMap = RwLock<HashMap<mime_guess::Mime, &'static str>>;

#[derive(Debug, Default)]
pub(crate) struct MimeMapper {
    mappings: MimeMap,
}

/// `MimeMapper` uses [`mime_guess`] to create a _Content-Type_ header and
/// stores it once in a global static [`HashMap`].
impl MimeMapper {
    /// This returns a global static instance
    #[inline]
    pub(crate) fn instance() -> &'static Self {
        &MIME_MAPPINGS
    }

    pub(crate) fn get_or_insert(&self, mime: mime_guess::Mime) -> &'static str {
        if let Ok(m) = self.mappings.read() {
            if let Some(ct) = m.get(&mime) {
                return ct;
            }
        }

        if let Ok(mut m) = self.mappings.write() {
            if let Some(ct) = m.get(&mime) {
                return ct;
            }
            let ct: &'static str =
                Box::leak(("Content-Type: ".to_string() + mime.as_ref()).into_boxed_str());
            let _ = m.insert(mime, ct);
            return ct;
        }

        CT_APPLICATION_OCTET_STREAM
    }
}

/// Alternative possibility without [`mime_guess`] and creating the roughly
/// half dozen required possibilities in a custom matcher.  
/// Needs to be extended for the required mime types.
#[inline]
#[allow(dead_code)]
pub(crate) fn mime_map(path: &str) -> &'static str {
    match path
        .rsplit_once('.')
        .unwrap_or(("", ""))
        .1
        .to_lowercase()
        .as_str()
    {
        "html" => "Content-Type: text/html; charset=utf-8",
        "jpg" | "jpeg" => "Content-Type: image/jpeg",
        "png" => "Content-Type: image/png",
        "svg" => "Content-Type: image/svg+xml",
        "css" => "Content-Type: text/css",
        "js" => "Content-Type: text/javascript",
        "txt" => "Content-Type: text/plain",
        "xml" => "Content-Type: text/xml",
        _ => CT_APPLICATION_OCTET_STREAM,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_map_test() {
        let extensions = ["jpg", "png", "svg", "xml", "html", "txt", "js", "css"];

        for (n, ext) in extensions.iter().enumerate() {
            let ct = mime_map(&("some/a/b/file.".to_string() + ext));
            let ct_start = if n <= 2 { "image/" } else { "text/" };
            assert!(
                ct.starts_with(&("Content-Type: ".to_string() + ct_start)),
                "ct: {ext} != {ct}"
            );
        }

        assert_eq!(mime_map("some/a/b/file.bin"), CT_APPLICATION_OCTET_STREAM);
    }

    #[test]
    fn mime_mapper_test() {
        let mm = MimeMapper::instance();

        assert_eq!(MIME_MAPPINGS.mappings.read().unwrap().len(), 0);

        let extensions = ["jpg", "png", "svg", "xml", "html", "txt", "js", "css"];

        // inserts leaked &'static str content types for every Mime of extension
        for (n, ext) in extensions.iter().enumerate() {
            let _ = mm.get_or_insert(mime_guess::from_ext(ext).first_or_octet_stream());
            assert_eq!(MIME_MAPPINGS.mappings.read().unwrap().len(), n + 1);
        }

        // should use the existing stored &'static str of content types and
        // map shouldn't become bigger
        for ext in extensions {
            let _ = mm.get_or_insert(mime_guess::from_ext(ext).first_or_octet_stream());
            assert_eq!(
                MIME_MAPPINGS.mappings.read().unwrap().len(),
                extensions.len()
            );
        }

        let mut ct = mm.get_or_insert(mime_guess::from_ext(extensions[0]).first_or_octet_stream());
        assert_eq!(ct, "Content-Type: image/jpeg");
        ct = mm.get_or_insert(mime_guess::from_ext(extensions[7]).first_or_octet_stream());
        assert_eq!(ct, "Content-Type: text/css");

        ct = mm.get_or_insert(mime_guess::from_ext("bin").first_or_octet_stream());
        assert_eq!(ct, CT_APPLICATION_OCTET_STREAM);
    }
}
