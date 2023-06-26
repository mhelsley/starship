use super::{Context, Module, ModuleConfig};
use crate::module::ALL_MODULES;
use crate::modules;
use crate::segment::Segment;
use nu_ansi_term::{unstyle, AnsiGenericString, AnsiStrings};

use crate::configs::terminal::TerminalConfig;
use crate::formatter::StringFormatter;

fn handle_module<'a>(module: &str, context: &'a Context) -> Vec<Module<'a>> {
    let mut module_vec: Vec<Module> = Vec::new();

    if ALL_MODULES.contains(&module) {
        // Write out a module if it isn't disabled
        if !context.is_module_disabled_in_config(module) {
            module_vec.extend(modules::handle(module, context));
        }
    } else {
        log::debug!(
            "Expected title_format to contain value from {:?}. Instead received {}",
            ALL_MODULES,
            module,
        );
    }

    module_vec
}

/// Allows the user to manipulate the terminal's title
/// May someday also enable users to emit "markers" at interesting
/// points which may show up in scrollbars / automatic bookmark list,
/// etc.
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    let mut module = context.new_module("terminal");
    let config: TerminalConfig = TerminalConfig::try_load(module.config);

    if let Ok(formatter) = StringFormatter::new(&config.title_format) {
        let parsed = formatter
            .map_variables_to_segments(|module| {
                if context.is_module_disabled_in_config(module) {
                    None
                } else {
                    // Get segments from module
                    Some(Ok(handle_module(module, &context)
                        .into_iter()
                        .flat_map(|module| module.segments)
                        .collect::<Vec<Segment>>()))
                }
            })
            .parse(None, Some(&context));
        module.set_segments(match parsed {
            Ok(segments) => {
                // Strip the styling before putting it into a single
                // Title segment.
                let mut title = Vec::<Segment>::with_capacity(1);
                let content = unstyle(&AnsiStrings(
                    segments
                        .iter()
                        .map(|segment| segment.ansi_string())
                        .collect::<Vec<AnsiGenericString<'_, str>>>()
                        .as_slice(),
                ));
                title.push(Segment::title(content));
                title
            }
            Err(error) => {
                log::warn!("Error in module `terminal`: \n{}", error);
                return None;
            }
        });
    } else {
        return None;
    }
    Some(module)
}

#[cfg(test)]
mod tests {
    use crate::test::ModuleRenderer;
    use nu_ansi_term::Color;

    #[test]
    fn config_terminal() {
        let actual = ModuleRenderer::new("terminal").title("sample").collect();

        let expected = None;
        assert_eq!(expected, actual);
    }
}
