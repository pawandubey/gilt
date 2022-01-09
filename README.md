# gilt (WIP)

A lazy tool for lazy people. Runs a given command across all your local git repositories.

> **⚠ Still a work in progress! Use at your own peril.**

## What

This is a simple tool that does a very simple thing: it takes a command and runs it in each git repository under a directory you point it to (default: `$HOME`). It then collates the result and prints them out per repository. That's it.

In addition, it has some quality-of-life options:
- [] (WIP) Colorize (slightly) the default output.
- [] (WIP) Output the result as JSON in case you want to pipe it into another program like `jq`.

## How

```bash
gilt --colorize --location /path/to/parent/dir --exec "git status -s" --output json
```

## Why

This started as a Ruby script that identified all unsynced branches across all local git repositories. Of course, I was not happy with such a specific solution, so had to go ahead and make something more generic so that I can never use it again.

## Why "gilt"?

Because "global git" and it's better than "glit", which was what I started with. I believe in quitting while you're ahead.

## License

Copyright Pawan Dubey.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
