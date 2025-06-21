<!-- cargo-rdme start -->

Basic SNSS file parsing (eg. Chrome Session and Tabs Files)

# Examples
```rust
let data = std::fs::read("Session")?;
let snss = snss::parse(&data)?;
for command in snss.commands {
    if let snss::Content::Tab(tab) = command.content {
        println!("Tab #{}: [{}]({})", tab.id, tab.title, tab.url);
    }
}
```

<!-- cargo-rdme end -->
