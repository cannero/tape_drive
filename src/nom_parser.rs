use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{alphanumeric1, multispace0},
    multi::many_till,
    sequence::delimited,
    IResult,
};

const CHANNEL_START_ID: &str = "- [Twitch](";

#[derive(Debug, PartialEq)]
pub struct Streamer {
    name: String,
    channel: String,
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
    let (input, _) = channel_start(input)?;
    let (input, channel) = streamer_channel(input)?;
    let (input, _) = streamer_start(input)?;
    Ok((
        input,
        Streamer {
            name: name.to_string(),
            channel: channel.to_string(),
        },
    ))
}

fn streamer_name(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    //TODO use preceded
    let (input, _) = alt((tag("### "), tag("## ")))(input)?;
    alphanumeric1(input)
}

fn streamer_channel(input: &str) -> IResult<&str, &str> {
    delimited(tag(CHANNEL_START_ID), take_while(|c| c != ')'), tag(")\n"))(input)
}

fn streamer_start(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_until("---")(input)?;
    alt((tag("----\n"), tag("---\n---\n"), tag("---\n")))(input)
}

fn channel_start(input: &str) -> IResult<&str, &str> {
    take_until(CHANNEL_START_ID)(input)
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
    fn test_parse_channel() {
        let input = "- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)

[(top)](#table-of-contents)

---";

        let (_, channel) = streamer_channel(input).unwrap();

        assert_eq!(channel, "https://www.twitch.tv/brookzerker");
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

        let (channel_start, _) = channel_start(input).unwrap();

        assert_eq!(
            channel_start,
            "- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)"
        );
    }

    #[test]
    fn test_parse_streamer() {
        let input = "### Brookzerker
#### What Brookzerker streams:
- Rust
#### Streaming on:
- [Twitch](https://www.twitch.tv/brookzerker)
#### Links:
- [Twitter](https://twitter.com/brooks_patton)
- [GitHub](https://github.com/BrooksPatton)

[(top)](#table-of-contents)

---
### Btor
#### What Btor streams:";

        let (_, streamer) = parse_streamer(input).unwrap();

        assert_eq!(
            streamer,
            Streamer {
                name: "Brookzerker".to_string(),
                channel: "https://www.twitch.tv/brookzerker".to_string(),
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
}
