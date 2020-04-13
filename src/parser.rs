use pulldown_cmark::{CowStr, Event, LinkType, Parser, Tag};

pub fn parse_md(text: &str) -> Result<(), String> {
    let parser = Parser::new(text).skip_while(|event| match event {
        Event::Text(CowStr::Borrowed("Developers That Stream")) => false,
        _ => true,
    });

    let parser = parser.skip_while(|event| match event {
        Event::Start(Tag::Heading(_)) => false,
        _ => true,
    });

    let streamer = parse_one_block(parser)?;
    println!("{:?}", streamer);
    Ok(())
}

#[derive(Debug)]
struct Streamer {
    name: String,
    topics: Vec<String>,
    link: String,
}

fn parse_one_block<'a>(mut parser: impl Iterator<Item = Event<'a>>) -> Result<Streamer, String> {
    parser.next();
    let name = match parser.next() {
        Some(Event::Text(CowStr::Borrowed(name))) => name,
        s => return Err(format!("not name, but {:?}", s)),
    };
    let mut parser = parser.skip_while(|event| match event {
        Event::Start(Tag::Item) => false,
        _ => true,
    });
    parser.next();
    let topics = match parser.next() {
        Some(Event::Text(CowStr::Borrowed(topics))) => topics,
        s => return Err(format!("not topics, but {:?}", s)),
    };

    let mut parser = parser.skip_while(|event| match event {
        Event::Start(Tag::Item) => false,
        _ => true,
    });
    parser.next();
    let link = match parser.next() {
        Some(Event::Start(Tag::Link(LinkType::Inline, CowStr::Borrowed(link), _))) => link,
        s => return Err(format!("not link, but {:?}", s)),
    };

    Ok(Streamer {
        name: name.to_string(),
        topics: topics.split(", ").map(str::to_string).collect(),
        link: link.to_string(),
    })
}
