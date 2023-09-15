# Carabiner Tech

The goal of Carabiner Tech is to connect AI models with coding environments in a safe and lightweight way. The problem we are trying to solve is allowing the LLM to iterate on problems with real-world feedback so that it can fix syntax mistakes, and work until builds compile or tests pass. In order to do that, we make a limited set of RPC commands available to the LLM while giving Users full control over what runtime those commands are executed in.

One way to think about human-computer interactions is to consider what *artifacts* you are left with after a conversation. At its simplest, that *artifact* is a text chat log. There might be valuable information in that *artifact*, such as code snippets you can copy/paste into an editor to run. With Carabiner, we want the *artifacts* to be a file system and git history of the operations the LLM took to get from the start of the conversation to the end, and we want the LLM to iterate on its own without you the user copy/pasting between Chat UI and your runtime.

# Overview


![Overview Diagram](./docs/diagrams/overview.svg)


# Try it out

If you have plugin developer access to ChatGPT, you can try out running our demo server and starting an Agent yourself.

1. Clone this repo, [setup Rust](https://www.rust-lang.org/tools/install) if necessary.
2. Start the demo server: `cd demo-server && cargo run`. It should show it's listening on `127.0.0.1:3000`.
3. In ChatGPT, [enable plugins](https://help.openai.com/en/articles/7183286-how-to-access-plugins), open the plugin store, and choose "develop your own plugin" at the bottom if you have that option (requires plugin developer access).
4. Enter `localhost:3000`, you'll see server logs of ChatGPT pulling the manifest file and openapi schema.
5. Start a conversation with the plugin enabled, perhaps ask it "What can the Carabiner plugin do?"
6. Start an Agent, `cd agent && cargo run`. You'll see the Agent's session id printed out in both Agent and Server logs.
7. Tell ChatGPT "use Agent <session id> for our conversation". You should see ChatGPT perform a `use_agent` operation (POST `/use_agent/:agent_id`).
8. Ask ChatGPT what the system time is on the Agent, or to list files at a directory. Watch what endpoints / operations the LLM chooses to use.
9. Experiment by adding new endpoints to `demo-server/src/api/mod.rs` to support other RPC operations. 

Note the "openai plugin devtools" is very useful to reload your locally-developed plugin here. Otherwise you'll need to uninstall / redevelop the plugin for ChatGPT to pick up changes to your OpenAPI schema.

# Coming Soon

 - An unverified and prod plugin you can install in ChatGPT to work with our hosted server
 - Docker containers for various runtimes with the Agent installed
 - Python / Rust libraries for openai function calling that interacts with a Carabiner server
 - Integration with other LLMs