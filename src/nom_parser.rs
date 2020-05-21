use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::multispace0,
    multi::many_till,
    sequence::{delimited, tuple},
    IResult,
};

const CHANNEL_START_ID: &str = "- [Twitch](";

#[derive(Debug, PartialEq)]
pub struct Streamer {
    name: String,
    login: String,
    content: String,
}

impl Streamer {
    pub fn login_name(&self) -> &String {
        &self.login
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

pub fn parse_file(input: &str) -> Result<Vec<Streamer>, String> {
    let (input, _) = streamers_block_start(input).unwrap();
    match parse_streamers(input) {
        Ok((_, (streamers, _))) => Ok(streamers),
        Err(e) => Err(format! {"Failed {}", e}),
    }
}

fn parse_streamers(input: &str) -> IResult<&str, (Vec<Streamer>, &str)> {
    many_till(parse_streamer, tag("\n## Twitch"))(input)
}

fn streamers_block_start(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_until("# Developers That Stream")(input)?;
    tag("# Developers That Stream\n\n")(input)
}

fn parse_streamer(input: &str) -> IResult<&str, Streamer> {
    let (input, name) = streamer_name(input)?;

    if vec![
        "Brad Garropy",
        "Coding Garden with CJ",
        "Daniel Shiffman",
        "DJ Adams",
        "Eddie Jaoude",
        "Gynvael Coldwind",
        "Jesse Weigel",
        "Josh Wulf",
        "Lizzie Siegle",
        "Luke Gorrie",
        "Masood Sadri",
        "Sallar Kaboli",
        "SkillVid",
        "Tanya Janca",                       //aka.ms, not using twitch any more?
        "Calvin Allen / Coding with Calvin", //streams on twitch, link re-directs
    ]
    .contains(&name)
    {
        //take next streamer, no twitch account
        let (input, _) = streamer_start(input)?;
        return parse_streamer(input);
    }

    let (input, _) = content_start(input)?;
    let (input, content) = stream_content(input)?;

    let channel_start_id = if name == "Randall Hunt" {
        "- [Twitch (Personal)]("
    } else {
        CHANNEL_START_ID
    };
    let (input, _) = channel_start(input, channel_start_id)?;
    let (input, login) = streamer_login(input, channel_start_id)?;
    let (input, _) = streamer_start(input)?;
    Ok((
        input,
        Streamer {
            name: name.trim().to_string(),
            login: login.trim_matches('/').to_string(),
            content: content.trim_matches('-').trim().to_string(),
        },
    ))
}

fn streamer_name(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    delimited(
        alt((tag("### "), tag("## "))),
        take_while(|c| c != '\n'),
        tag("\n"),
    )(input)
}

fn streamer_login<'a>(input: &'a str, channel_start_id: &str) -> IResult<&'a str, &'a str> {
    delimited(
        tuple((
            tag(channel_start_id),
            alt((tag("https:"), tag("http:"))),
            alt((tag("//www.twitch.tv/"), tag("//twitch.tv/"))),
        )),
        take_while(|c| c != ')'),
        tag(")\n"),
    )(input)
}

fn streamer_start(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_until("--")(input)?;
    alt((tag("----\n"), tag("---\n---\n"), tag("---\n"), tag("--\n")))(input)
}

fn channel_start<'a>(input: &'a str, channel_start_id: &str) -> IResult<&'a str, &'a str> {
    let (input, taken) = take_until(channel_start_id)(input)?;
    if taken.contains("---") {
        eprintln!("--- found");
        Err(nom::Err::Error((taken, nom::error::ErrorKind::OneOf)))
    } else {
        Ok((input, taken))
    }
}

fn content_start(input: &str) -> IResult<&str, &str> {
    let content_headline_start = "#### What ";
    let (input, taken) = take_until(content_headline_start)(input)?;
    if taken.contains("--") {
        eprintln!("-- found during content start search");
        return Err(nom::Err::Error((taken, nom::error::ErrorKind::OneOf)));
    }

    let (input, _) = tag(content_headline_start)(input)?;
    let (input, _) = take_until("\n")(input)?;
    tag("\n")(input)
}

fn stream_content(input: &str) -> IResult<&str, &str> {
    take_until("\n")(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streamer_block_find_start() {
        let input = "some text here
another text
text and more text
- [Zorchenhimer](#zorchenhimer) **streaming** 6502 NES Assembly, Golang

[(top)](#table-of-contents)

----
# Developers That Stream

### Adam13531
#### What Adam streams:
";
        let (rest, _) = streamers_block_start(input).unwrap();

        assert_eq!(
            rest,
            "### Adam13531
#### What Adam streams:
"
        );
    }

    #[test]
    fn test_parse_streamer_name() {
        let input = "### Brookzerker
#### What Brookzerker streams:
- Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)

[(top)](#table-of-contents)

---";

        let (_, streamer_name) = streamer_name(input).unwrap();

        assert_eq!(streamer_name, "Brookzerker");
    }

    #[test]
    fn test_parse_streamer_name_whitespace_before() {
        let input = "
### britocoding
#### What britocoding
";

        let (_, streamer_name) = streamer_name(input).unwrap();

        assert_eq!(streamer_name, "britocoding");
    }

    #[test]
    fn test_parse_login() {
        let input = "- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)

[(top)](#table-of-contents)

---";

        let (_, login) = streamer_login(input, CHANNEL_START_ID).unwrap();

        assert_eq!(login, "brookzerker");
    }

    #[test]
    fn test_parse_login_only_http() {
        let input = "- [Twitch](http://twitch.tv/Shinmera)
";

        let (_, login) = streamer_login(input, CHANNEL_START_ID).unwrap();

        assert_eq!(login, "Shinmera");
    }

    #[test]
    fn test_streamer_start() {
        let input = "#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)

[(top)](#table-of-contents)

---
### Btor
#### What Btor streams:";

        let (streamer_start, _) = streamer_start(input).unwrap();

        assert_eq!(
            streamer_start,
            "### Btor
#### What Btor streams:"
        );
    }

    #[test]
    fn test_streamer_start_four_dashes() {
        let input = "[(top)](#table-of-contents)

----
### Frank Boucher";

        let (streamer_start, _) = streamer_start(input).unwrap();

        assert_eq!(streamer_start, "### Frank Boucher");
    }

    #[test]
    fn test_channel_start() {
        let input = "#### What Brookzerker streams:
- Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)";

        let (channel_start, _) = channel_start(input, CHANNEL_START_ID).unwrap();

        assert_eq!(
            channel_start,
            "- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)"
        );
    }

    #[test]
    fn test_channel_start_two_twitch_channels() {
        let input = "#### What Randall streams:
AWS, Web Development, Python, Serverless, AI
#### Streaming on:
- [Twitch (AWS)](https://www.twitch.tv/aws)
- [Twitch (Personal)](https://www.twitch.tv/RandallAtAmazon)
#### Links:
- [Twitter](https://twitter.com/jrhunt)
- [GitHub](https://github.com/ranman)
- [YouTube](https://www.youtube.com/channel/UC-yKovfbYEWyD_pXh9n7nHA)

[(top)](#table-of-contents)

---
### Ricardo Tavares
#### What Ricardo streams:
Angular 6+, SCSS, LUA, Node.js, Python, SQL, Typescript, WASM, Web Development
#### Streaming on:
- [Mixer](https://mixer.com/Rjgtav)
- [Twitch](https://www.twitch.tv/rjgtav/)
- [YouTube](https://www.youtube.com/user/rjgtav)";

        let start = channel_start(input, CHANNEL_START_ID);

        assert!(start.is_err());
    }

    #[test]
    fn test_parse_streamer() {
        let input = "### Mike Conley
#### What Mike streams:
- Firefox Development, JavaScript, C++, CSS, Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/mikeconley_dot_ca/)
- [Facebook](https://www.facebook.com/TheJoyOfCoding1/)
- [YouTube](https://www.youtube.com/channel/UCTDXvmarLFnox4AO0w2NuiQ)
- [Air Mozilla](https://air.mozilla.org/channels/livehacking/)
#### Links:
- [Twitter](http://twitter.com/mike_conley)
- [GitHub](http://github.com/mikeconley/)
- [YouTube](https://www.youtube.com/channel/UCTDXvmarLFnox4AO0w2NuiQ)
- [Website](https://www.mikeconley.ca/blog)

[(top)](#table-of-contents)

---
### Nicholas Brochu
#### What Nicholas streams:
- Python, Serpent.AI Framework Dev, Machine Learning, AI, Computer Vision
";

        let (_, streamer) = parse_streamer(input).unwrap();

        assert_eq!(
            streamer,
            Streamer {
                name: "Mike Conley".to_string(),
                login: "mikeconley_dot_ca".to_string(),
                content: "Firefox Development, JavaScript, C++, CSS, Rust".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_streamers() {
        let input = "### WindybeardGames
#### What Windy streams:
- ClickTeam Fusion, PhotoShop, Spine, Game Art
#### Languages Spoken During Stream
- English
- Wincy
#### Streaming on:
- [Twitch](https://www.twitch.tv/windybeardgames)
#### Links:
- [Discord](https://discord.gg/FKumdQY)
- [Patreon](https://www.patreon.com/Windybeardgames)
- [Twitter](https://twitter.com/WindybeardGames)
- [Website](http://windybeard.com/)

[(top)](#table-of-contents)

---
### Yosh
#### What Yosh streams:
- OSS maintenance, Tooling Development, JavaScript, Node.js, Choo
#### Streaming on:
- [Twitch](https://www.twitch.tv/yoshuawuyts)
#### Links:
- [Twitter](https://twitter.com/yoshuawuyts)
- [GitHub](https://github.com/yoshuawuyts)

[(top)](#table-of-contents)

---
### Zorchenhimer
#### What Zorchenhimer streams:
- 6502 NES Assembly, Golang
#### Streaming on:
- [Twitch](https://www.twitch.tv/zorchenhimer)
#### Links:
- [Twitter](https://twitter.com/Zorchenhimer)
- [GitHub](https://github.com/zorchenhimer)
- [Website](https://zorchenhimer.com/)
- [Youtube](https://www.youtube.com/c/Zorchenhimer)

---
---

## Twitch";

        let (_, (streamers, _)) = parse_streamers(input).unwrap();

        assert_eq!(streamers.len(), 3);
    }

    #[test]
    fn test_content_start() {
        let input = "#### What Brookzerker streams:
- Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)";

        let (content, _) = content_start(input).unwrap();

        assert_eq!(
            content,
            "- Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)"
        );
    }

    #[test]
    fn test_stream_content() {
        let input = "Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)";

        let (_, content) = stream_content(input).unwrap();

        assert_eq!(content, "Rust");
    }

    #[test]
    fn test_content_start_without_minus() {
        let input = "#### What Holden Karau streams:
Scala, Python, Spark, PySpark, Open Source
#### Streaming on:
- [Twitch](https://www.twitch.tv/holdenkarau)
#### Languages Spoken During Stream
- English
";

        let (content, _) = content_start(input).unwrap();

        assert_eq!(
            content,
            "Scala, Python, Spark, PySpark, Open Source
#### Streaming on:
- [Twitch](https://www.twitch.tv/holdenkarau)
#### Languages Spoken During Stream
- English
"
        );
    }

    #[test]
    fn test_stream_content_without_minus() {
        let input = "Scala, Python, Spark, PySpark, Open Source
#### Streaming on:
- [Twitch](https://www.twitch.tv/holdenkarau)
#### Languages Spoken During Stream
- English
";

        let (_, content) = stream_content(input).unwrap();

        assert_eq!(content, "Scala, Python, Spark, PySpark, Open Source");
    }
}
