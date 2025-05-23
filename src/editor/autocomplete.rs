use typst::syntax::Source;
use typst_ide::{Completion, CompletionKind, IdeWorld};

/// Generates autocompletion suggestions for a given position in the source code.
///
/// Returns a tuple of the start position and a list of Typst [`Completion`] suggestions, if available.
pub fn autocomplete(
    world: &dyn IdeWorld,
    source: &Source,
    cursor: usize,
) -> Option<(usize, Vec<Completion>)> {
    let (pos, completions) = typst_ide::autocomplete(
        world, None, //temp?
        source, cursor, true,
    )?;
    println!("autocompletion starts at {}", pos);
    println!(
        "labels: {:?}\n",
        completions
            .iter()
            .map(|c| c.label.as_str())
            .collect::<Vec<_>>()
    );
    //log_completions(&completions);

    let start = source.get(pos..cursor)?;

    if !start.is_empty() && start.ne("") {
        return complete_word(start, &completions).map(|c| (pos, c));
    }

    Some((pos, completions))
}

/// Filters completion suggestions based on the current word prefix.
///
/// Returns the list of matching completions, if any.
fn complete_word(word: &str, available: &Vec<Completion>) -> Option<Vec<Completion>> {
    let mut completions = vec![];
    for completion in available {
        if completion.label.starts_with(word) {
            completions.push(completion.clone());
        }
    }

    log_completions(&completions);

    Some(completions)
}

/// Logs the available completion suggestions for debugging purposes.
fn log_completions(completions: &Vec<Completion>) {
    println!("{} completions", completions.len());
    for completion in completions {
        match completion.kind {
            CompletionKind::Syntax => (),
            _ => {
                println!("Kind: {:?}", completion.kind);
                println!("Label: {:?}", completion.label);
                println!("Apply: {:?}", completion.apply);
                println!("Details: {:?}", completion.detail);
                println!("\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world;
    use crate::world::TideWorld;
    use iced::widget::text_editor::Content;
    use typst::World;

    trait Assertion {
        fn labels(&self) -> Vec<&str>;
        fn includes<'a>(&self, includes: impl IntoIterator<Item = &'a str>) -> &Self;
        fn excludes<'a>(&self, excludes: impl IntoIterator<Item = &'a str>) -> &Self;
        fn expects_len(&self, len: usize) -> &Self;
    }

    type Completions = Option<Vec<Completion>>;

    impl Assertion for Completions {
        fn labels(&self) -> Vec<&str> {
            match self {
                Some(completions) => completions.iter().map(|x| x.label.as_str()).collect(),
                None => vec![],
            }
        }

        fn includes<'a>(&self, includes: impl IntoIterator<Item = &'a str>) -> &Self {
            let labels = self.labels();
            for elem in includes {
                assert!(labels.contains(&elem), "{elem:?} not in {labels:?}");
            }

            self
        }

        fn excludes<'a>(&self, excludes: impl IntoIterator<Item = &'a str>) -> &Self {
            let labels = self.labels();
            for elem in excludes {
                assert!(!labels.contains(&elem), "{elem:?} in {labels:?}");
            }

            self
        }

        fn expects_len(&self, len: usize) -> &Self {
            let labels = self.labels();
            assert_eq!(labels.len(), len);

            self
        }
    }

    fn change_main(world: &mut TideWorld, content: &str, shift: usize) -> (Source, usize) {
        let main = world.main();
        let content = Content::with_text(content);
        world.reload_source_from_content(main, &content);
        let source = world.source(main).unwrap();
        let cursor = source.len_bytes() - 1; //EOF

        (source, cursor - shift)
    }

    struct TestWorld {
        world: TideWorld,
        shift: usize,
    }

    impl TestWorld {
        fn init() -> Self {
            Self {
                world: world::tests::init_world(),
                shift: 0,
            }
        }

        fn left_shift(&mut self, shift: usize) -> &mut Self {
            self.shift = shift;

            self
        }

        fn test_word(&mut self, main_content: &str) -> Completions {
            let (source, cursor) = change_main(&mut self.world, main_content, self.shift);

            let (pos, completions) =
                typst_ide::autocomplete(&self.world, None, &source, cursor, true)
                    .unwrap_or((0, vec![]));

            let word = source.get(pos..cursor)?;
            let result = if word.is_empty() || word.eq("") {
                None
            } else {
                complete_word(word, &completions)
            };
            self.shift = 0;

            result
        }

        fn test_completion(&mut self, main_content: &str) -> Completions {
            let (source, cursor) = change_main(&mut self.world, main_content, self.shift);
            let result = autocomplete(&self.world, &source, cursor).map(|(_, com)| com);
            self.shift = 0;

            result
        }
    }

    #[test]
    fn test_complete_filter() {
        let mut world = TestWorld::init();
        world
            .test_word("#ima")
            .includes(["image"])
            .excludes(["label", "expression", "linebreak"])
            .expects_len(1);
        world
            .test_word("ima")
            .includes([])
            .excludes(["image"])
            .expects_len(0);
        world
            .test_word("")
            .includes([])
            .excludes(["label", "expression"])
            .expects_len(0);
        world
            .test_word("#f")
            .includes(["float", "function", "figure"])
            .excludes(["list", "array"])
            .expects_len(8);
        world
            .test_word("#fi")
            .includes(["figure"])
            .excludes(["float", "function"])
            .expects_len(1);
        world
            .test_word("#sym.arrow.b")
            .includes(["b", "bar", "bl", "br"])
            .excludes(["curve", "dashed"])
            .expects_len(4);
        world
            .test_word("#sym.arrow.")
            .includes([])
            .excludes(["bar", "curve", "dashed", "half"])
            .expects_len(0);
        world
            .left_shift(1)
            .test_word("#figure()")
            .includes([])
            .excludes(["image", "caption"])
            .expects_len(0);
    }

    #[test]
    fn test_complete_general() {
        let mut world = TestWorld::init();
        world
            .test_completion("#sym.arrow.")
            .includes(["b", "bar", "bl", "br", "curve", "dashed"])
            .excludes(["expression"])
            .expects_len(32);
        world
            .left_shift(1)
            .test_completion("#figure()")
            .includes(["image", "caption"])
            .excludes(["list", "array"])
            .expects_len(12);
        world
            .test_completion("")
            .includes(["label", "expression", "linebreak"])
            .expects_len(16)
            .excludes(["figure"]);
        world
            .left_shift(6)
            .test_completion("#figure()")
            .includes(["figure"])
            .expects_len(1)
            .excludes(["image", "caption"]);
        world
            .left_shift(1)
            .test_completion("#text(font:)")
            .includes(["\"Libertinus Math\"", "\"Libertinus Serif\""]); //can't expect the number of fonts
        world
            .test_completion("ima")
            .includes(["label", "expression"])
            .excludes(["image"])
            .expects_len(16);
    }
}
