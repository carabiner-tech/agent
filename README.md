# Carabiner Tech

The goal of Carabiner Tech is to connect AI models with coding environments in a safe and lightweight way. The problem we are trying to solve is allowing the LLM to iterate on problems with real-world feedback so that it can fix syntax mistakes, and work until builds compile or tests pass. In order to do that, we make a limited set of RPC commands available to the LLM while giving Users full control over what runtime those commands are executed in.

One way to think about human-computer interactions is to consider what *artifacts* you are left with after a conversation. At its simplest, that *artifact* is a text chat log. There might be valuable information in that *artifact*, such as code snippets you can copy/paste into an editor to run. With Carabiner, we want the *artifacts* to be a file system and git history of the operations the LLM took to get from the start of the conversation to the end, and we want the LLM to iterate on its own without you the user copy/pasting between Chat UI and your runtime.

# Overview


![Overview Diagram](./docs/diagrams/overview.svg)