# Crodot

This file is an example of a project description and the **actual** project description.

Crodot is a terminal LLM coding assistant. It is, by design, very simple. The user is in complete control, ie not agentic. It contains the following features:

- some convenience methods for generating prompts. This includes:
  - easy ways to add file contents to prompts
  - automated generation of the file structure of the (source) dir
- you can prompt in two ways:
  - question: ask an LLM for help, explanations or checks of the code in your repo
  - coding: ask the LLM to write code for you repo

That is it. The app is a simple wrapper around this functionality.

The user should bring 2 different sets of settings. First, a general config that includes which LLM to use. And second, per project configs that point to the source dir, active files, etc.

The project is written in **rust**, making heavy use of the **ratatui** library.

Please note that until now all code was written by a rust amateur, which will show in the quality and consistency of the code.
