import { workspace, ExtensionContext, window } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export async function activate(context: ExtensionContext): Promise<void> {
  const config = workspace.getConfiguration("sage");
  const sagePath = config.get<string | null>("path") || "sage";

  // sage sense communicates over stdio
  const serverOptions: ServerOptions = {
    command: sagePath,
    args: ["sense"],
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    // Activate for all .sg files
    documentSelector: [{ scheme: "file", language: "sage" }],
    // Watch sage.toml for project configuration changes
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/sage.toml"),
    },
    traceOutputChannel: window.createOutputChannel("Sage LSP Trace"),
  };

  client = new LanguageClient(
    "sage",
    "Sage Language Server",
    serverOptions,
    clientOptions
  );

  await client.start();
}

export async function deactivate(): Promise<void> {
  await client?.stop();
}
