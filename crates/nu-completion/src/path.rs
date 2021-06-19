use std::borrow::Cow;
use std::path::{is_separator, Path, PathBuf};

use super::matchers::Matcher;
use crate::{Completer, CompletionContext, Suggestion};

const SEP: char = std::path::MAIN_SEPARATOR;

pub struct PathCompleter;


#[derive(Debug)]
pub struct PathSuggestion {
    pub(crate) path: PathBuf,
    pub(crate) suggestion: Suggestion,
}

impl PathCompleter {
    pub fn path_suggestions(&self, partial: &str, matcher: &dyn Matcher) -> Vec<PathSuggestion> {
        dbg!(&partial);
        let (base_dir_name, partial) = {
            let (base, rest) = partial.rsplit_once(is_separator).unwrap_or((".", partial));
            // On windows, this standardizes paths to use \
            let mut base = base.replace(is_separator, &SEP.to_string());

            // rsplit_once removes the separator
            base.push(SEP);
            (base, rest)
        };

        let base_dir = nu_path::expand_path(Cow::Borrowed(Path::new(&base_dir_name)));
        dbg!(&base_dir);
        if !base_dir.is_dir() {
            return Vec::new();
        }

        let r = if let Ok(result) = base_dir.read_dir() {
            result
                .filter_map(|entry| {
                    entry.ok().and_then(|entry| {
                        let mut file_name = entry.file_name().to_string_lossy().into_owned();
                        if matcher.matches(partial, file_name.as_str()) {
                            let mut path = format!("{}{}", &base_dir_name, file_name);
                            dbg!(&path);
                            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                                path.push(SEP);
                                file_name.push(SEP);
                            }

                            Some(PathSuggestion {
                                path: entry.path(),
                                suggestion: Suggestion {
                                    replacement: path,
                                    display: file_name,
                                },
                            })
                        } else {
                            None
                        }
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        dbg!(r)
    }
}

impl<Context> Completer<Context> for PathCompleter
where
    Context: CompletionContext,
{
    fn complete(&self, _ctx: &Context, partial: &str, matcher: &dyn Matcher) -> Vec<Suggestion> {
        self.path_suggestions(partial, matcher)
            .into_iter()
            .map(|ps| ps.suggestion)
            .collect()
    }
}
