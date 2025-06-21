# snss

Basic SNSS file parsing (eg. Chrome Session and Tabs Files)

## Examples
```rust
let data = std::fs::read("~/.config/vivaldi/Default/Sessions/Session_13395009830123502")?;
let snss = snss::parse(&data)?;
for command in snss.commands {
    if let snss::Content::Tab(tab) = command.content {
        println!("Tab #{}: [{}]({})", tab.id, tab.title, tab.url);
    }
}
```

License: GPL-3.0-or-later
