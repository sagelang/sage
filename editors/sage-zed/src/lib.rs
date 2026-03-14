use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

struct SageExtension;

impl zed::Extension for SageExtension {
    fn new() -> Self {
        SageExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        // Look up `sage` on the user's PATH
        let path = worktree.which("sage").ok_or_else(|| {
            "sage binary not found on PATH. \
             Install Sage from https://sagelang.dev/install \
             or set `lsp.sage-sense.binary.path` in Zed settings."
                .to_string()
        })?;

        Ok(Command {
            command: path,
            args: vec!["sense".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(SageExtension);
