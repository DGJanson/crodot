# Crodot

Crodot is a lightweight, non-agentic terminal LLM coding assistant built in Rust using `ratatui`.

## Features

- **Prompt Generation**: Easily bundle source file structures and contents into LLM prompts.
- **Dual Mode Operation**:
  - **Question Mode**: Ask questions, request code explanations, or get reviews.
  - **Coding Mode**: Have the LLM return structured code changes (new files or edits) to apply directly to your repository.
- **Config Management**: Global LLM configuration and project-specific settings.

## Crodot is NOT:

Very user friendly.

Agentic.

Feature rich.

A code or text editor.

## Motivation

Like many, I find LLM's very useful for programming. I have been using it either in chat mode (in browser) or fully integrated in VS code (codex). But as someone who liked to do some coding manually sometimes, I was basically looking for some middle ground between a chat window in a browser and a fully integrated (agentic) solution. I could not really find something that suited me, so tried to create it myself. Crodot is the result. It works somewhat for myself. But it probably mostly works just for me...

## Intended use

Crodot is a tui app meant to be used in conjunction with a code editor. In crodot you interact with an LLM, asking questions about your code, request code reviews or ask it to write code. The main way to interact with your code is in your code editor, not directly in crodot.

The basis (and hopefully usefulness) of crodot comes from simple prompt manipulation and a quick way to interact with the LLM results. This makes for quick feedback loops on the one hand and a decent amount of control for the programmer (you) on the other hand.

Crodot starts from a project definition. In this project you define: a root dir, a project description (for the LLM) and a list of files you want to add to the context. This forms the basis of the prompts and interaction with the LLM. The prompt consists of:

- the project description (txt or md)
- the source dirs structure; you can manage which directories are included
- the file **contents** of active files you have added

This allows for quick interaction about your code with the LLM. In the prompt screen you can select two prompt modes: Question and Coding. Question is like a normal chat, you will get a response that you can read. It is possible to quickly yank coding blocks from this response. Coding leads to a structured response with changes to files. You can for example ask to create a new file with a new function or ask to edit a file that you have already created. You handle the Coding response by reviewing the LLM output files and deciding if you want to write them to your repo.

## How to run

For now, this crodot repo only contains the code. You can simply build and run it using cargo. You need a json config file to start the app, pass it as a argument to crodot. Eg.

```
cargo run ~/configs/crodot-config.json  
```

The config file has settings for LLM providers, like urls, which env_var holds the key, etc. (TODO -> add an example config file).

## TODO list

- [x] Documentation on how to use the app *(first version in 0.2.0)*
- [ ] Add example config file
- [x] Quick way to select and copy code blocks from the Question Response *(added in 0.2.0)*
- [x] Option to add more source dirs (or rather add tests dir and a documentation dir possibly). *(added in 0.2.0)*
- [ ] Allow for more LLM providers in config.
- [ ] Move this list to a more sensible location
