# tenorcli
tenorcli is a cli program for fetching from the [tenor api](https://developers.google.com/tenor/guides/quickstart)<br>
With tenorcli you can search, copy links, and/or save gifs to your computer from the commandline<br>

### Installing and updating
run `cargo install --git https://github.com/ethanp4/tenorcli.git`

### Compiling
run `cargo build`

### Setup
Before starting you must provide your own free api key obtained from https://developers.google.com/tenor/guides/quickstart<br>
Set it using `tenorcli --set-api-key <key>`

### Usage examples:
 - `tenorcli` (equivalent to tenorcli -t file -l10 cat) -- list 10 tenor page links
 - `tenorcli --limit 15 yakuza goro majima watermelon` -- list 15 gif links
 - `tenorcli -l5 -cq kitten good morning` -- copy a random link from the first 5 results, with no output
 - `tenorcli -t file -r nano-gif dog` -- copy a random link from the first 10 results with a very small resolution
